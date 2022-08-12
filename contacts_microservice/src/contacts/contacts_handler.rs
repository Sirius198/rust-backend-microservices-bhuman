use anyhow::Context;
use axum::{
    async_trait,
    extract::{rejection::JsonRejection, Query},
    Extension, Json,
};
use axum_macros::debug_handler;
use openapi_rs::openapi_proc_macro::handler;
use serde_json::json;
use serde_json::Value;
use shopify::customer::Customer;
use shopify::customer_address::CustomerAddress;
use shopify::order::{Currency, Order};
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Utc;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use std::fmt::Write;
use tonic::{Code, Status};

use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;
use openapi_rs::OpenApiFromData;

use crate::{
    contacts::{
        contacts::{ContactQuery, ContactRes, ContactSync},
    },
};
use microservice_utils::server::response::{AxumRes,into_reponse, AxumResult};
use microservice_utils::server::grpc::{get_shopify_token};
use microservice_utils::jwt::extractor::AuthToken;

use crate::contacts::contacts::{Contact, ContactModel, Provider, GenericContact, GoogleContacts, OutlookContacts};

pub mod address_book_service {
    tonic::include_proto!("address_book_service");
}

use address_book_service::address_book_service_server::AddressBookService;
use address_book_service::{
    ShopifyDataRequest, ShopifyDataResponse, ShopifyOrder, ShopifyOrdersRedact, ShopifyResponse,
};

pub struct MyAddressBookService {
    pool: PgPool,
}

impl MyAddressBookService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AddressBookService for MyAddressBookService {
    async fn push_shopify_order(
        &self,
        request: tonic::Request<ShopifyOrder>,
    ) -> Result<tonic::Response<ShopifyResponse>, tonic::Status> {
        let req: ShopifyOrder = request.into_inner();
        println!("Adding Shopify Order {:?}", req);

        let _ = insert_shopify_order(&self.pool, &req.into())
            .await
            .with_context(|| anyhow::anyhow!("failed to insert shopify order"))
            .map_err(|e| Status::new(Code::Internal, format!("{:?}", e)))?;

        Ok(tonic::Response::new(ShopifyResponse {
            status: "Ok".to_string(),
        }))
    }

    async fn request_shopify_data(
        &self,
        request: tonic::Request<ShopifyDataRequest>,
    ) -> Result<tonic::Response<ShopifyDataResponse>, tonic::Status> {
        let req: ShopifyDataRequest = request.into_inner();
        println!("Requesting Shopify Orders {:?}", req);

        let customer_id = req.customer_id.map(|x| {
            std::str::from_utf8(&x.value)
                .map_or("", |f| f)
                .parse::<i64>()
                .expect("failed to parse customer id")
        });

        let contacts = request_shopify_data(&self.pool, customer_id, req.requested_orders)
            .await
            .with_context(|| anyhow::anyhow!("failed to request shopify orders"))
            .map_err(|e| Status::new(Code::Internal, format!("{:?}", e)))?;

        Ok(tonic::Response::new(ShopifyDataResponse {
            status: "Ok".to_string(),
            contacts: contacts.iter().map(|f| f.clone().into()).collect(),
        }))
    }

    async fn redact_shopify_orders(
        &self,
        request: tonic::Request<ShopifyOrdersRedact>,
    ) -> Result<tonic::Response<ShopifyResponse>, tonic::Status> {
        let req: ShopifyOrdersRedact = request.into_inner();
        println!("Deleting Shopify Orders {:?}", req);

        let customer_id = req.customer_id.map(|x| {
            std::str::from_utf8(&x.value)
                .map_or("", |f| f)
                .parse::<i64>()
                .expect("failed to parse customer id")
        });

        let _ = delete_shopify_orders(&self.pool, customer_id, req.orders_to_redact)
            .await
            .with_context(|| anyhow::anyhow!("failed to delete shopify orders"))
            .map_err(|e| Status::new(Code::Internal, format!("{:?}", e)))?;

        Ok(tonic::Response::new(ShopifyResponse {
            status: "Ok".to_string(),
        }))
    }
}

#[debug_handler]
#[handler(method = "POST",tag = "address_book")]
pub async fn sync_contacts(
    payload: Result<Json<ContactSync>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let sync_info = payload.0;
            println!("Received token {:?}", sync_info);

            let client = reqwest::Client::new();

            match sync_info.provider {
                Provider::Google => {
                    let person_fields = String::from("addresses,birthdays,emailAddresses,genders,names,organizations,phoneNumbers,photos,userDefined");
                    let mut next = String::from("start");
                    let mut total = 0;
                    let mut first: Vec<GenericContact> = Vec::new();
                    while next.len() > 0 {
                        let mut url = "https://people.googleapis.com/v1/people/me/connections".to_string();
                        write!(url, "?personFields={}", person_fields).unwrap();
                        write!(url, "&pageSize={}", 1000).unwrap();

                        if next != "start" {
                            write!(url, "&pageToken={}", next).unwrap();
                        }                       

                        let request = client
                        .get(url)
                        .header("Content-Type", "application/json")
                        .header("Authorization", format!("Bearer {}", sync_info.token))
                        .send()
                        .await
                        .map_err(|e| into_reponse(500, e.to_string().into()))?;

                        let contacts = request
                            .json::<GoogleContacts>()
                            .await
                            .map_err(|e| into_reponse(500, e.to_string().into()))?;

                        let _ = sync_google_contacts(
                            &user_id,
                            &sync_info.phone,
                            &sync_info.email,
                            &contacts,
                            &pool,
                        )
                        .await
                        .map_err(|e| into_reponse(500, e.to_string().into()))?;   
                        
                        total += contacts.contacts.len();

                        if next == "start" {
                            let mut count = total;
                            if count > 10 {
                                count = 10;
                            }
                            first.extend(contacts.contacts[0..count].to_vec());
                        }

                        if let Some(n) = contacts.next {
                            next = n;
                        } else {
                            next = "".to_string();
                        }
                    }
                    
                    let response: ContactRes = ContactRes {
                        user_id: user_id,
                        phone: sync_info.phone,
                        email: sync_info.email,
                        provider: sync_info.provider,
                        total: total,
                        contacts: json!(first),
                    };
                    
                    let add_contacts = axum::Json(AxumRes {
                        result: json!(response),
                        code: 200,
                    });

                    Ok(add_contacts)
                }
                Provider::Outlook => {
                    let person_fields = String::from("givenName,surname,emailAddresses,mobilePhone");
                    let mut next = String::from("start");
                    let mut skip = 0;                    
                    let mut total = 0;
                    let mut first: Vec<GenericContact> = Vec::new();
                    while next.len() > 0 {
                        let mut url = "https://graph.microsoft.com/v1.0/me/contacts".to_string();
                        write!(url, "?personFields={}", person_fields).unwrap();

                        if next != "start" {
                            write!(url, "&skip={}", skip).unwrap();
                        }

                        let request = client
                            .get(url)
                            .header("Content-Type", "application/json")
                            .header("Authorization", format!("Bearer {}", sync_info.token))
                            .send()
                            .await
                            .map_err(|e| into_reponse(500, e.to_string().into()))?;

                        let contacts = request
                            .json::<OutlookContacts>()
                            .await
                            .map_err(|e| into_reponse(500, e.to_string().into()))?;

                        let _ = sync_outlook_contacts(
                            &user_id,
                            &sync_info.phone,
                            &sync_info.email,
                            &contacts,
                            &pool,
                        )
                        .await
                        .map_err(|e| into_reponse(500, e.to_string().into()))?;

                        total += contacts.contacts.len();

                        if next == "start" {
                            first.extend(contacts.contacts[0..total].to_vec());
                        }

                        if let Some(n) = contacts.next {
                            next = n;
                            skip += 10;
                        } else {
                            next = "".to_string();
                        }                        
                    }

                    let response: ContactRes = ContactRes {
                        user_id: user_id,
                        phone: sync_info.phone,
                        email: sync_info.email,
                        provider: sync_info.provider,
                        total: total,
                        contacts: json!(first),
                    };
                    
                    let add_contacts = axum::Json(AxumRes {
                        result: json!(response),
                        code: 200,
                    });

                    Ok(add_contacts)
                }
                Provider::Shopify => {
                    // let shopify_token: String;
                    let shopify_token = get_shopify_token(&user_id.to_string())
                        .await
                        .map_err(|e| into_reponse(500, e.to_string().into()))?;

                    let request = client
                        .get("https://shopify.bhuman.ai/api/customers/fetch_customers")
                        // .get("http://localhost:3004/api/customers/fetch_customers")
                        .header("Content-Type", "application/json")
                        .bearer_auth(shopify_token)
                        .header(reqwest::header::USER_AGENT, "curl/7.64.1")
                        .send()
                        .await
                        .map_err(|e| into_reponse(500, e.to_string().into()))?;

                    let contacts = request
                        .json::<Vec<Contact>>()
                        .await
                        .map_err(|e| into_reponse(500, e.to_string().into()))?;

                    let add_contacts = sync_shopify_contacts(
                        &user_id,
                        &sync_info.phone,
                        &sync_info.email,
                        &contacts,
                        &pool,
                    )
                    .await
                    .map_err(|e| into_reponse(500, e.to_string().into()))
                    .map(|_| {
                        let response: ContactRes = ContactRes {
                            user_id: user_id,
                            phone: sync_info.phone,
                            email: sync_info.email,
                            provider: sync_info.provider,
                            total: contacts.len(),
                            contacts: json!(contacts),
                        };
                        axum::Json(AxumRes {
                            result: json!(response),
                            code: 200,
                        })
                    })?;

                    Ok(add_contacts)
                }
                Provider::DefaultProvider => {
                    let ret = serde_json::json!({
                        "error": "Provider not found",
                    });
                    Err(into_reponse(400, ret))
                }
            }
        }
        Err(e) => {
            println!("{:?}", e.to_string());
            let ret = serde_json::json!({
                "error": format!("{:?}", e),
            });
            Err(into_reponse(400, ret))
        }
    }
}

#[debug_handler]
#[handler(method = "GET", tag = "address_book", description = "", summary = "")]
pub async fn get_contacts(
    params: Query<ContactQuery>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match &params.provider {
        Provider::Google => {
            let contacts = get_generic_contacts(&user_id, &params, &pool)
                .await
                .map_err(|e| into_reponse(500, e.to_string().into()))?;

            Ok(axum::Json(AxumRes {
                result: json!(contacts),
                code: 200,
            }))
        }
        Provider::Outlook => {
            let contacts = get_generic_contacts(&user_id, &params, &pool)
                .await
                .map_err(|e| into_reponse(500, e.to_string().into()))?;
            Ok(axum::Json(AxumRes {
                result: json!(contacts),
                code: 200,
            }))
        }
        Provider::Shopify => {
            let contacts = get_shopify_contacts(&user_id, &pool)
                .await
                .map_err(|e| into_reponse(500, e.to_string().into()))?;
            Ok(axum::Json(AxumRes {
                result: json!(contacts),
                code: 200,
            }))
        }
        Provider::DefaultProvider => {
            println!("{:?}", "provider not found");
            let ret = serde_json::json!({
                "error": "provider not found",
            });
            Err(into_reponse(500, ret))
        }
    }
}

// Database
pub async fn sync_google_contacts(
    user_id: &String,
    phone: &String,
    email: &String,
    contacts: &GoogleContacts,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        "INSERT INTO contacts (user_id, phone, email) VALUES ($1, $2, $3) ON CONFLICT (user_id) DO NOTHING",
        user_id,
        phone,
        email,
    )
    .execute(pool)
    .await?;

    for contact in &contacts.contacts {
        let _ = sqlx::query!(
            "INSERT INTO generic_contacts (identifier, user_id, provider, name, photo, phone_numbers, email_addresses) VALUES ($1, $2, $3, $4, $5, $6, $7) 
            ON CONFLICT (identifier) DO UPDATE SET name = $4, photo = $5, phone_numbers = $6, email_addresses = $7",
            contact.identifier,
            user_id,
            "google".to_string(),
            contact.name,
            contact.photo,
            &contact.phone_numbers.as_ref().map_or(Vec::new(), |f| f.to_vec()),
            &contact.email_addresses.as_ref().map_or(Vec::new(), |f| f.to_vec()),
        ).execute(pool).await?;
    }
    Ok(())
}

pub async fn sync_shopify_contacts(
    user_id: &String,
    phone: &String,
    email: &String,
    contacts: &Vec<Contact>,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        "INSERT INTO contacts (user_id, phone, email) VALUES ($1, $2, $3) ON CONFLICT (user_id) DO NOTHING",
        user_id,
        phone,
        email,
    )
    .execute(pool)
    .await?;

    for contact in contacts {
        let addresses = &contact.customer_addresses;
        let orders = &contact.customer_orders;

        let _ = sqlx::query!(
            "INSERT INTO shopify_contacts (customer_id,user_id,customer_phone,customer_email,
                accepts_marketing,
                note,
                currency,
                created_at,
                updated_at,
                first_name,
                last_name,
                orders_count,
                total_spent,
                last_order_id,
                verified_email,
                tax_exempt) VALUES ($1, $2, $3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)",
            contact.customer.id,
            user_id,
            contact.customer.phone,
            contact.customer.email,
            contact.customer.accepts_marketing,
            contact
                .customer
                .note
                .as_ref()
                .map_or(String::new(), |f| f.to_string()),
            contact.customer.currency,
            DateTime::<Utc>::from_str(&contact.customer.created_at)
                .expect("failed to parse date from contacts.customer.created_at")
                .naive_utc(),
            DateTime::<Utc>::from_str(&contact.customer.updated_at)
                .expect("failed to parse date from contacts.customer.updated_at")
                .naive_utc(),
            contact.customer.first_name,
            contact.customer.last_name,
            contact.customer.orders_count as i32,
            contact.customer.total_spent,
            contact
                .customer
                .last_order_id
                .as_ref()
                .map_or(0, |f| if let Value::Number(n) = f {
                    n.as_i64().map_or(0, |x| x)
                } else {
                    0
                }) as i32,
            contact.customer.verified_email,
            contact.customer.tax_exempt
        )
        .execute(pool)
        .await?;

        for address in addresses {
            let _ = sqlx::query!(
                "INSERT INTO shopify_customer_addresses (address_customer_id,address1,address2,
                    city,
                    country,
                    country_code,
                    country_name,
                    company,
                    province,
                    province_code,
                    zip) VALUES ($1, $2, $3,$4,$5,$6,$7,$8,$9,$10,$11)",
                address.customer_id,
                address.address1,
                address.address2,
                address.city,
                address.country,
                address.country_code,
                address.country_name,
                address.company,
                address.province,
                address.province_code,
                address.zip
            )
            .execute(pool)
            .await?;
        }

        for order in orders {
            let _ = insert_shopify_order(pool, order).await?;
        }
    }

    Ok(())
}

pub async fn sync_outlook_contacts(
    user_id: &String,
    phone: &String,
    email: &String,
    contacts: &OutlookContacts,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        "INSERT INTO contacts (user_id, phone, email) VALUES ($1, $2, $3) ON CONFLICT (user_id) DO NOTHING",
        user_id,
        phone,
        email,
    )
    .execute(pool)
    .await?;

    for contact in &contacts.contacts {
        let _ = sqlx::query!(
            "INSERT INTO generic_contacts (identifier, user_id, provider, name, photo, phone_numbers, email_addresses) VALUES ($1, $2, $3, $4, $5, $6, $7) 
            ON CONFLICT (identifier) DO UPDATE SET name = $4, photo = $5, phone_numbers = $6, email_addresses = $7",
            contact.identifier,
            user_id,
            "outlook".to_string(),
            contact.name,
            contact.photo,
            &contact.phone_numbers.as_ref().map_or(Vec::new(), |f| f.to_vec()),
            &contact.email_addresses.as_ref().map_or(Vec::new(), |f| f.to_vec()),
        ).execute(pool).await?;
    }
    Ok(())
}

pub async fn get_shopify_contacts(
    user_id: &String,
    pool: &PgPool,
) -> Result<ContactRes, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT contacts.phone,contacts.email,shopify_contacts.*,shopify_customer_addresses.*,shopify_customer_orders.* FROM contacts INNER JOIN shopify_contacts
        ON contacts.user_id = shopify_contacts.user_id INNER JOIN shopify_customer_orders ON shopify_customer_orders.order_customer_id = shopify_contacts.customer_id INNER JOIN shopify_customer_addresses ON shopify_customer_addresses.address_customer_id = shopify_contacts.customer_id  WHERE contacts.user_id = $1;",
        user_id
    )
    .fetch_all(pool)
    .await?;
    let mut phone = String::new();
    if rows[0].phone.is_some() {
        phone = rows[0].phone.as_ref().unwrap().to_string();
    }
    let mut email = String::new();
    if rows[0].email.is_some() {
        email = rows[0].email.as_ref().unwrap().to_string();
    }

    let user_id = rows[0].user_id.clone();

    let mut contacts = vec![];

    for row in rows {
        let shopify_contact = Customer {
            id: row.customer_id.into(),
            accepts_marketing: row.accepts_marketing,
            orders_count: row.orders_count.into(),
            phone: row.customer_phone.clone(),
            last_order_id: row.last_order_id.map(|f| f.into()),
            first_name: row.first_name.clone(),
            last_name: row.last_name.clone(),
            currency: row.currency,
            verified_email: row.verified_email,
            created_at: row.created_at.to_string(),
            updated_at: row.updated_at.to_string(),
            email: row.customer_email.clone(),
            tax_exempt: row.tax_exempt,
            total_spent: row.total_spent,
            ..Default::default()
        };

        let shopify_contact_address = CustomerAddress {
            address1: row.address1,
            address2: row.address2,
            city: row.city,
            country: row.country,
            country_code: row.country_code,
            country_name: row.country_name,
            company: row.company,
            customer_id: row.address_customer_id.into(),
            first_name: row.first_name.clone(),
            id: row.address_id.into(),
            last_name: row.last_name.clone(),
            phone: row.customer_phone.clone(),
            province: row.province,
            province_code: row.province_code,
            zip: row.zip,
            name: String::new(),
        };

        let currency = match row.order_currency.as_str() {
            "USD" => Currency::Usd,
            "CAD" => Currency::Cad,
            "EUR" => Currency::Eur,
            _ => Currency::Usd,
        };

        let shopify_contact_order = Order {
            id: row.order_id.into(),
            app_id: row.app_id.into(),
            browser_ip: row.browser_ip.map(|f| Value::String(f)),
            buyer_accepts_marketing: row.buyer_accepts_marketing.map_or(false, |f| f),
            cancel_reason: row.cancel_reason.map(|f| Value::String(f)),
            cancelled_at: row.cancelled_at.map(|f| Value::String(f.to_string())),
            cart_token: row.cart_token.map(|f| Value::String(f)),
            checkout_id: row.checkout_id.map(|f| Value::Number(f.into())),
            closed_at: row.closed_at.map(|f| Value::String(f.to_string())),
            confirmed: row.confirmed,
            contact_email: row.contact_email,
            created_at: row.created_at.to_string(),
            currency: currency,
            current_subtotal_price: row.current_subtotal_price,
            current_total_discounts: row.current_total_discounts,
            current_total_price: row.current_total_price,
            current_total_tax: row.current_total_tax,
            device_id: row.device_id.map(|f| Value::String(f)),
            email: row.customer_email,
            financial_status: row.financial_status,
            fulfillment_status: row.fulfillment_status.map(|f| Value::String(f)),
            name: row.order_name,
            note: row.order_note.map(|f| Value::String(f)),
            number: row.customer_number.into(),
            order_number: row.order_number.into(),
            phone: Some(Value::String(row.customer_phone)),
            processed_at: row.processed_at.to_string(),
            processing_method: row.processing_method,
            subtotal_price: row.subtotal_price,
            total_price: row.total_price,
            total_price_usd: row.total_price_usd,
            total_tax: row.total_tax,
            updated_at: row.updated_at.to_string(),
            ..Default::default()
        };

        contacts.push(Contact {
            customer: shopify_contact,
            customer_orders: vec![shopify_contact_order],
            customer_addresses: vec![shopify_contact_address],
        });
    }

    let res = ContactRes {
        user_id: user_id,
        phone: phone,
        email: email,
        total: contacts.len(),
        provider: Provider::Shopify,
        contacts: json!(contacts),
    };
    Ok(res)
}

pub async fn request_shopify_data(
    pool: &PgPool,
    customer_id: Option<i64>,
    orders: Vec<i64>,
) -> Result<Vec<ContactModel>, sqlx::Error> {
    let mut orders_str = String::new();
    for order in orders {
        orders_str.push_str(&format!("{},", order));
    }
    orders_str.pop();

    let query = match customer_id {
        Some(id) => format!(
            "SELECT * FROM shopify_contacts sc INNER JOIN shopify_customer_orders sco ON sco.order_customer_id = sc.customer_id 
            WHERE sc.customer_id = {} AND sco.order_id IN ({})",
            id,orders_str
        ),
        None => format!(
            "SELECT * FROM shopify_customer_orders WHERE order_id IN ({})",
            orders_str
        ),
    };

    let contact = sqlx::query_as::<_, ContactModel>(&query)
        .fetch_all(pool)
        .await?;
    Ok(contact)
}

pub async fn delete_shopify_orders(
    pool: &PgPool,
    customer_id: Option<i64>,
    orders: Vec<i64>,
) -> Result<(), sqlx::Error> {
    let mut orders_str = String::new();
    for order in orders {
        orders_str.push_str(&format!("{},", order));
    }
    orders_str.pop();

    let query = match customer_id {
        Some(id) => format!(
            "DELETE FROM shopify_customer_orders WHERE order_id IN ({}) AND customer_id = {}",
            orders_str, id
        ),
        None => format!(
            "DELETE FROM shopify_customer_orders WHERE order_id IN ({})",
            orders_str
        ),
    };

    let _ = sqlx::query(&query).execute(pool).await?;
    Ok(())
}

pub async fn insert_shopify_order(pool: &PgPool, order: &Order) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        "INSERT INTO shopify_customer_orders (order_id,order_customer_id,
            app_id,
            browser_ip,
            buyer_accepts_marketing,
            cancel_reason,
            cancelled_at,
            cart_token,
            checkout_id,
            closed_at,
            confirmed,
            contact_email,
            order_created_at,
            order_currency,
            current_subtotal_price,
            current_total_discounts,
            current_total_duties_set,
            current_total_price,
            current_total_tax,
            device_id,
            order_email,
            financial_status,
            fulfillment_status,
            order_name,
            customer_number,
            order_number,
            order_note,
            processed_at,
            processing_method,
            subtotal_price,
            total_price,
            total_tax,
            total_price_usd,
            order_updated_at,
            line_items) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31,$32,$33,$34,$35)",
            order.id,
            order.customer.id,
            order.app_id as i32,
            order.browser_ip.as_ref()
            .map_or(String::new(), |f| f.to_string()),
            order.buyer_accepts_marketing,
            order.cancel_reason.as_ref()
            .map_or(String::new(), |f| f.to_string()),
            order.cancelled_at.as_ref().map(|f| DateTime::<Utc>::from_str(&f.to_string()).expect("failed to parse date").naive_utc()),
            order.cart_token.as_ref()
            .map_or(String::new(), |f| f.to_string()),
            order.checkout_id.as_ref()
            .map_or(0, |f| if let Value::Number(n) = f {
                n.as_i64().map_or(0, |x| x)
            } else {
                0
            }) as i32,
            order.closed_at.as_ref().map(|f| DateTime::<Utc>::from_str(&f.to_string()).expect("failed to parse date").naive_utc()),
            order.confirmed,
            order.contact_email,
            DateTime::<Utc>::from_str(&order.created_at).expect("failed to parse date").naive_utc(),
            json!(order.currency).to_string(),
            order.current_subtotal_price,
            order.current_total_discounts,
            order.current_total_duties_set.as_ref()
            .map_or(String::new(), |f| f.to_string()),
            order.current_total_price,
            order.current_total_tax,
            order.device_id.as_ref()
            .map_or(String::new(), |f| f.to_string()),
            order.email,
            order.financial_status,
            order.fulfillment_status.as_ref()
            .map_or(String::new(), |f| f.to_string()),
            order.name,
            order.phone.as_ref()
            .map_or(0, |f| if let Value::Number(n) = f {
                n.as_i64().map_or(0, |x| x)
            } else {
                0
            }) as i32,
            order.order_number as i32,
            order.note.as_ref()
            .map_or(String::new(), |f| f.to_string()),
            DateTime::<Utc>::from_str(&order.processed_at).expect("failed to parse date").naive_utc(),
            order.processing_method,
            order.subtotal_price,
            order.total_price,
            order.total_tax,
            order.total_price_usd,
            DateTime::<Utc>::from_str(&order.updated_at).expect("failed to parse date").naive_utc(),
            String::new()
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_generic_contacts(
    user_id: &String,
    params: &ContactQuery,
    pool: &PgPool,
) -> Result<ContactRes, sqlx::Error> {

    let contact = sqlx::query!("SELECT * FROM contacts WHERE user_id = $1", user_id).fetch_one(pool).await?;
    let row = sqlx::query!("SELECT count(*) OVER() AS total FROM generic_contacts WHERE user_id = $1 AND provider = $2", user_id, params.provider.to_string().to_lowercase()).fetch_one(pool).await?;
    let total = row.total.unwrap();

    let mut query = "SELECT *, count(*) OVER() AS total FROM generic_contacts WHERE ".to_string();
    write!(query, "user_id = '{}' AND provider = '{}' ", user_id, params.provider.to_string().to_lowercase()).unwrap();

    if let Some(q) = &params.query {
        if q.len() > 0 {
            write!(query, "AND (LOWER(name) LIKE LOWER('%{}%') OR ARRAY_TO_STRING(phone_numbers, ',') LIKE '%{}%' OR ARRAY_TO_STRING(email_addresses, ',') LIKE '%{}%') LIMIT 50", 
            q, q, q).unwrap();
        }        
    } else {
        write!(query, "OFFSET {} LIMIT {}", params.page * params.size, params.size).unwrap();
    }
    
    let str_query: &str = &query[..];
    let contacts = sqlx::query_as::<_, GenericContact>(str_query).fetch_all(pool).await?;

    let res = ContactRes {
        user_id: user_id.to_string(),
        phone: contact.phone.map_or(String::new(), |f| f.to_string()),
        email: contact.email.map_or(String::new(), |f| f.to_string()),
        provider: params.provider.clone(),
        total: total as usize,
        contacts: json!(contacts),
    };
    Ok(res)
}

pub async fn get_generic_contact_by_identifier(
    user_id: &String,
    identifier: &String,
    pool: &PgPool,
) -> Result<GenericContact, sqlx::Error> {
    let row = sqlx::query!("SELECT * FROM generic_contacts WHERE user_id = $1 AND identifier = $2", user_id, identifier).fetch_one(pool).await?;
    let contact = GenericContact {
        identifier: row.identifier,
        name: row.name,
        photo: row.photo,
        phone_numbers: row.phone_numbers,
        email_addresses: row.email_addresses,
    };
    Ok(contact)
}

