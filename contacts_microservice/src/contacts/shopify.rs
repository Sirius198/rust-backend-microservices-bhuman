use std::str::FromStr;

use prost_types::{Timestamp,Any};
use serde_json::json;
use shopify::{order::{Order, Currency}, customer::Address};
use sqlx::types::chrono::{DateTime, NaiveDateTime, Utc};

use super::{contacts_handler::address_book_service::{ShopifyOrder, shopify_data_response::{Contacts, DefaultAddress, EmailMarketingConsent, SmsMarketingConsent, Addresses}, shopify_order::Customer}, contacts::ContactModel};

impl Into<Order> for ShopifyOrder {
    fn into(self) -> Order {
        let currency = match self.currency.as_str() {
            "USD" => Currency::Usd,
            "CAD" => Currency::Cad,
            "EUR" => Currency::Eur,
            _ => Currency::Usd,
        };

        let presentment_currency = match self.presentment_currency.as_str() {
            "USD" => Currency::Usd,
            "CAD" => Currency::Cad,
            "EUR" => Currency::Eur,
            _ => Currency::Usd,
        };

        let created_at = self.created_at.map_or(Timestamp::default(), |f| f);

        let updated_at = self.updated_at.map_or(Timestamp::default(), |f| f);

        let processed_at = self.processed_at.map_or(Timestamp::default(), |f| f);

        Order {
            id: self.id,
            email: self.email,
            name: self.name,
            phone: Some(json!(self.phone)),
            total_price: self.total_price,
            currency: currency,
            created_at: DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(created_at.seconds, created_at.nanos as u32),
                Utc,
            )
            .to_string(),
            updated_at: DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(updated_at.seconds, updated_at.nanos as u32),
                Utc,
            )
            .to_string(),
            device_id: self
                .device_id
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            admin_graphql_api_id: self.admin_graphql_api_id,
            app_id: self.app_id,
            browser_ip: self
                .browser_ip
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            buyer_accepts_marketing: self.buyer_accepts_marketing,
            cancel_reason: self
                .cancel_reason
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            cancelled_at: self
                .cancelled_at
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            cart_token: self
                .cart_token
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            checkout_id: self
                .checkout_id
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            checkout_token: self
                .checkout_token
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            closed_at: self
                .closed_at
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            confirmed: self.confirmed,
            contact_email: self.contact_email,
            current_subtotal_price: self.current_subtotal_price,
            current_total_discounts: self.current_total_discounts,
            current_total_duties_set: self
                .current_total_duties_set
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            current_total_price: self.current_total_price,
            current_total_tax: self.current_total_tax,
            customer_locale: self
                .customer_locale
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            discount_codes: self
                .discount_codes
                .iter()
                .map(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f))))
                .collect(),
            estimated_taxes: self.estimated_taxes,
            financial_status: self.financial_status,
            fulfillment_status: self
                .fulfillment_status
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            gateway: self.gateway,
            landing_site: self
                .landing_site
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            landing_site_ref: self
                .landing_site_ref
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            location_id: self
                .location_id
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            note: Some(json!(self.note)),
            note_attributes: self
                .note_attributes
                .iter()
                .map(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f))))
                .collect(),
            number: self.number,
            order_number: self.order_number,
            order_status_url: self.order_status_url,
            original_total_duties_set: self
                .original_total_duties_set
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            payment_gateway_names: self
                .payment_gateway_names
                .iter()
                .map(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f))))
                .collect(),
            presentment_currency: presentment_currency,
            processed_at: DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(processed_at.seconds, processed_at.nanos as u32),
                Utc,
            )
            .to_string(),
            processing_method: self.processing_method,
            reference: self
                .reference
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            referring_site: self
                .referring_site
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            source_identifier: self
                .source_identifier
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            source_name: self.source_name,
            source_url: self
                .source_url
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            subtotal_price: self.subtotal_price,
            tags: self.tags,
            tax_lines: self
                .tax_lines
                .iter()
                .map(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f))))
                .collect(),
            taxes_included: self.taxes_included,
            test: self.test,
            token: self.token,
            total_discounts: self.total_discounts,
            total_line_items_price: self.total_line_items_price,
            total_outstanding: self.total_outstanding,
            total_price_usd: self.total_price_usd,
            total_tax: self.total_tax,
            total_tip_received: self.total_tip_received,
            total_weight: self.total_weight,
            user_id: Some(json!(self.order_user_id)),
            discount_applications: self
                .discount_applications
                .iter()
                .map(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f))))
                .collect(),
            fulfillments: self
                .fulfillments
                .iter()
                .map(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f))))
                .collect(),
            payment_terms: self
                .payment_terms
                .and_then(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f)))),
            refunds: self
                .refunds
                .iter()
                .map(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f))))
                .collect(),
            shipping_lines: self
                .shipping_lines
                .iter()
                .map(|x| Some(json!(std::str::from_utf8(&x.value).map_or("", |f| f))))
                .collect(),
            ..Default::default()
        }
    }
}

impl Into<ShopifyOrder> for Order {
    fn into(self) -> ShopifyOrder {
        let currency = match self.currency {
            Currency::Usd => "USD".to_string(),
            Currency::Cad => "CAD".to_string(),
            Currency::Eur => "EUR".to_string(),
        };

        let presentment_currency = match self.presentment_currency {
            Currency::Usd => "USD".to_string(),
            Currency::Cad => "CAD".to_string(),
            Currency::Eur => "EUR".to_string(),
        };

        let created_at = DateTime::<Utc>::from_str(&self.created_at)
            .ok()
            .map(|f| Timestamp {
                seconds: f.timestamp(),
                nanos: f.timestamp_subsec_nanos() as i32,
            });

        let updated_at = DateTime::<Utc>::from_str(&self.updated_at)
            .ok()
            .map(|f| Timestamp {
                seconds: f.timestamp(),
                nanos: f.timestamp_subsec_nanos() as i32,
            });

        let processed_at = DateTime::<Utc>::from_str(&self.processed_at)
            .ok()
            .map(|f| Timestamp {
                seconds: f.timestamp(),
                nanos: f.timestamp_subsec_nanos() as i32,
            });

        let device_id = self.device_id.and_then(|f| {
            Some(Any {
                type_url: "type.googleapis.com/address_book_service.AddShopifyOrder".to_string(),
                value: f.to_string().into_bytes(),
            })
        });

        ShopifyOrder {
            id: self.id,
            email: self.email,
            name: self.name,
            phone: self.phone.map_or(String::new(), |f| f.to_string()),
            total_price: self.total_price,
            currency: currency,
            created_at: created_at,
            updated_at: updated_at,
            device_id: device_id,
            admin_graphql_api_id: self.admin_graphql_api_id,
            app_id: self.app_id,
            browser_ip: self.browser_ip.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            buyer_accepts_marketing: self.buyer_accepts_marketing,
            cancel_reason: self.cancel_reason.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            cancelled_at: self.cancelled_at.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            cart_token: self.cart_token.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            checkout_id: self.checkout_id.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            checkout_token: self.checkout_token.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            closed_at: self.closed_at.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            confirmed: self.confirmed,
            contact_email: self.contact_email,
            current_subtotal_price: self.current_subtotal_price,
            current_total_discounts: self.current_total_discounts,
            current_total_duties_set: self.current_total_duties_set.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            current_total_price: self.current_total_price,
            current_total_tax: self.current_total_tax,
            customer: Some(Customer {
                id: self.customer.id,
                email: self.customer.email,
                phone: self.customer.phone,
                ..Default::default()
            }),
            customer_locale: self.customer_locale.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            discount_codes: self
                .discount_codes
                .iter()
                .map(|x| Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: x
                        .as_ref()
                        .map_or(String::new(), |f| f.to_string())
                        .into_bytes(),
                })
                .collect(),
            estimated_taxes: self.estimated_taxes,
            financial_status: self.financial_status,
            fulfillment_status: self.fulfillment_status.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            gateway: self.gateway,
            landing_site: self.landing_site.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            landing_site_ref: self.landing_site_ref.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            location_id: self.location_id.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            note: self.note.map_or(String::new(), |f| f.to_string()),
            note_attributes: self
                .note_attributes
                .iter()
                .map(|x| Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: x
                        .as_ref()
                        .map_or(String::new(), |f| f.to_string())
                        .into_bytes(),
                })
                .collect(),
            number: self.number,
            order_number: self.order_number,
            order_status_url: self.order_status_url,
            original_total_duties_set: self.original_total_duties_set.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            payment_gateway_names: self
                .payment_gateway_names
                .iter()
                .map(|x| Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: x
                        .as_ref()
                        .map_or(String::new(), |f| f.to_string())
                        .into_bytes(),
                })
                .collect(),
            presentment_currency: presentment_currency,
            processed_at: processed_at,
            processing_method: self.processing_method,
            reference: self.reference.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            referring_site: self.referring_site.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            source_identifier: self.source_identifier.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            source_name: self.source_name,
            source_url: self.source_url.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            subtotal_price: self.subtotal_price,
            tags: self.tags,
            tax_lines: self
                .tax_lines
                .iter()
                .map(|x| Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: x
                        .as_ref()
                        .map_or(String::new(), |f| f.to_string())
                        .into_bytes(),
                })
                .collect(),
            taxes_included: self.taxes_included,
            test: self.test,
            token: self.token,
            total_discounts: self.total_discounts,
            total_line_items_price: self.total_line_items_price,
            total_outstanding: self.total_outstanding,
            total_price_usd: self.total_price_usd,
            total_tax: self.total_tax,
            total_tip_received: self.total_tip_received,
            total_weight: self.total_weight,
            user_id: self.user_id.map_or(String::new(), |f| f.to_string()),
            discount_applications: self
                .discount_applications
                .iter()
                .map(|x| Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: x
                        .as_ref()
                        .map_or(String::new(), |f| f.to_string())
                        .into_bytes(),
                })
                .collect(),
            fulfillments: self
                .fulfillments
                .iter()
                .map(|x| Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: x
                        .as_ref()
                        .map_or(String::new(), |f| f.to_string())
                        .into_bytes(),
                })
                .collect(),
            payment_terms: self.payment_terms.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            refunds: self
                .refunds
                .iter()
                .map(|x| Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: x
                        .as_ref()
                        .map_or(String::new(), |f| f.to_string())
                        .into_bytes(),
                })
                .collect(),
            shipping_lines: self
                .shipping_lines
                .iter()
                .map(|x| Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: x
                        .as_ref()
                        .map_or(String::new(), |f| f.to_string())
                        .into_bytes(),
                })
                .collect(),
            ..Default::default()
        }
    }
}

impl Into<Addresses> for Address {
    fn into(self) -> Addresses {
        Addresses {
            address1: self.address1,
            address2: self.address2.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            city: self.city,
            company: self.company.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            country: self.country,
            country_code: self.country_code,
            first_name: self.first_name,
            last_name: self.last_name,
            phone: self.phone,
            province: self.province,
            province_code: self.province_code,
            zip: self.zip,
            country_name: self.country_name,
            customer_id: self.customer_id,
            default: self.address_default,
            id: self.id,
            name: self.name,
        }
    }
}

impl Into<Contacts> for ContactModel {
    fn into(self) -> Contacts {

        let created_at = DateTime::<Utc>::from_str(&self.customer.created_at)
        .ok()
        .map(|f| Timestamp {
            seconds: f.timestamp(),
            nanos: f.timestamp_subsec_nanos() as i32,
        });

        let accepts_marketing_updated_at = DateTime::<Utc>::from_str(&self.customer.accepts_marketing_updated_at)
        .ok()
        .map(|f| Timestamp {
            seconds: f.timestamp(),
            nanos: f.timestamp_subsec_nanos() as i32,
        });

        let updated_at = DateTime::<Utc>::from_str(&self.customer.updated_at)
        .ok()
        .map(|f| Timestamp {
            seconds: f.timestamp(),
            nanos: f.timestamp_subsec_nanos() as i32,
        });

        Contacts {
            accepts_marketing: self.customer.accepts_marketing,
            accepts_marketing_updated_at: accepts_marketing_updated_at,
            addresses: self.customer.addresses.iter().map(|f| f.clone().into()).collect(),
            admin_graphql_api_id: self.customer.0.admin_graphql_api_id,
            created_at: created_at,
            currency: self.customer.0.currency,
            customer_orders: self.customer_orders.iter().map(|f| f.clone().into()).collect(),
            default_address: Some(DefaultAddress {
                address1: self.customer.0.default_address.address1,
                address2: self.customer.0.default_address.address2.and_then(|f| {
                    Some(Any {
                        type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                            .to_string(),
                        value: f.to_string().into_bytes(),
                    })
                }),
                city: self.customer.0.default_address.city,
                company: self.customer.0.default_address.company.and_then(|f| {
                    Some(Any {
                        type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                            .to_string(),
                        value: f.to_string().into_bytes(),
                    })
                }),
                country: self.customer.0.default_address.country,
                country_code: self.customer.0.default_address.country_code,
                country_name: self.customer.0.default_address.country_name,
                customer_id: self.customer.0.default_address.customer_id,
                default: self.customer.0.default_address.address_default,
                first_name: self.customer.0.default_address.first_name,
                id: self.customer.0.default_address.id,
                last_name: self.customer.0.default_address.last_name,
                name: self.customer.0.default_address.name,
                phone: self.customer.0.default_address.phone,
                province: self.customer.0.default_address.province,
                province_code: self.customer.0.default_address.province_code,
                zip: self.customer.0.default_address.zip,
            }),
            email: self.customer.0.email,
            email_marketing_consent: Some(EmailMarketingConsent {
                state: self.customer.0.email_marketing_consent.state,
                opt_in_level: self.customer.0.email_marketing_consent.opt_in_level,
                consent_updated_at: self.customer.0.email_marketing_consent.consent_updated_at.and_then(|f| {
                    Some(Any {
                        type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                            .to_string(),
                        value: f.to_string().into_bytes(),
                    })
                }),
                consent_collected_from: self.customer.0.email_marketing_consent.consent_collected_from.and_then(|f| {
                    Some(Any {
                        type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                            .to_string(),
                        value: f.into_bytes(),
                    })
                }),
            }),
            first_name: self.customer.0.first_name,
            id: self.customer.0.id,
            last_name:  self.customer.0.last_name,
            last_order_id: self.customer.0.last_order_id.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            last_order_name: self.customer.0.last_order_name.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            marketing_opt_in_level: self.customer.0.marketing_opt_in_level.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            multipass_identifier: self.customer.0.multipass_identifier.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            note: self.customer.0.note.and_then(|f| {
                Some(Any {
                    type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                        .to_string(),
                    value: f.to_string().into_bytes(),
                })
            }),
            orders_count: self.customer.0.orders_count,
            phone: self.customer.0.phone,
            sms_marketing_consent: Some(SmsMarketingConsent {
                consent_collected_from: self.customer.0.sms_marketing_consent.consent_collected_from.map_or(String::new(), |f| f),
                consent_updated_at: self.customer.0.sms_marketing_consent.consent_updated_at.and_then(|f| {
                    Some(Any {
                        type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                            .to_string(),
                        value: f.to_string().into_bytes(),
                    })
                }),
                opt_in_level: self.customer.0.sms_marketing_consent.opt_in_level,
                state: self.customer.0.sms_marketing_consent.state,
            }),
            state: self.customer.0.state,
            tags: self.customer.0.tags,
            tax_exempt: self.customer.0.tax_exempt,
            tax_exemptions: self.customer.0.tax_exemptions.iter()
            .map(|x| Any {
                type_url: "type.googleapis.com/address_book_service.AddShopifyOrder"
                    .to_string(),
                value: x
                    .as_ref()
                    .map_or(String::new(), |f| f.to_string())
                    .into_bytes(),
            })
            .collect(),
            total_spent: self.customer.0.total_spent,
            updated_at: updated_at,
            verified_email: self.customer.0.verified_email,
        }
    }
}
