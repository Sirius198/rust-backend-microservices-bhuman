use axum::extract::Query;
use axum_macros::FromRequest;

use openapi_rs::openapi_proc_macro::query;
use schemars::JsonSchema;
use okapi::openapi3::Parameter;
use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;
use serde::Deserialize;
use serde::Serialize;
use serde::Deserializer;

use serde_json::Value;
use shopify::{customer::Customer, customer_address::CustomerAddress, order::Order};
use sqlx::types::Json;
use sqlx::FromRow;
use std::fmt;

// Google
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NameInfo {
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "familyName")]
    pub family_name: Option<String>,
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhotoInfo {
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhoneInfo {
    pub value: Option<String>,
    #[serde(rename = "formattedType")]
    pub formatted_type: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailInfo {
    pub value: Option<String>,
    #[serde(rename = "formattedType")]
    pub formatted_type: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoogleContactInfo {
    #[serde(rename = "resourceName")]
    pub resource_name: String,
    pub names: Option<Vec<NameInfo>>,
    pub photos: Option<Vec<PhotoInfo>>,
    #[serde(rename = "phoneNumbers")]
    pub phone_numbers: Option<Vec<PhoneInfo>>,
    #[serde(rename = "emailAddresses")]
    pub email_addresses: Option<Vec<EmailInfo>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoogleContactList {
    pub connections: Vec<GoogleContactInfo>,    
    #[serde(rename = "nextPageToken")]
    pub next: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct GenericContact {
    pub identifier: String,
    pub name: Option<String>,
    pub photo: Option<String>,
    pub phone_numbers: Option<Vec<String>>,
    pub email_addresses: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct GoogleContacts {
    pub contacts: Vec<GenericContact>,
    pub next: Option<String>
}

impl<'de> Deserialize<'de> for GoogleContacts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let mut contacts: Vec<GenericContact> = Vec::new();
        let connection = GoogleContactList::deserialize(deserializer)?;
        for contact in connection.connections.iter() {
            
            let mut name = String::from("");
            if let Some(names) = &contact.names {
                name = names[0].display_name.as_ref().map_or(String::new(), |f| f.to_string());
            }

            let mut photo = String::from("");
            if let Some(photos) = &contact.photos {
                photo = photos[0].url.as_ref().map_or(String::new(), |f| f.to_string());
            }

            let mut phone_numbers = Vec::new();
            if let Some(numbers) = &contact.phone_numbers {
                for number in numbers {
                    phone_numbers.push(number.value.as_ref().map_or(String::new(), |f| f.to_string()));
                }
            }

            let mut email_addresses = Vec::new();
            if let Some(emails) = &contact.email_addresses {
                for email in emails {
                    email_addresses.push(email.value.as_ref().map_or(String::new(), |f| f.to_string()));
                }
            }

            let item = GenericContact {
                identifier: contact.resource_name.clone(),
                name: Some(name),
                photo: Some(photo),
                phone_numbers: Some(phone_numbers),
                email_addresses: Some(email_addresses),
            };
            contacts.push(item);
        }
        Ok(GoogleContacts { contacts: contacts, next: connection.next })
    }
}

// Outlook
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutlookEmailInfo {
    pub name: Option<String>,
    pub address: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutlookContactInfo {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,    
    #[serde(rename = "homePhones")]
    pub home_phones: Option<Vec<String>>,
    #[serde(rename = "mobilePhone")]
    pub mobile_phone: Option<String>,
    #[serde(rename = "businessPhones")]
    pub business_phones: Option<Vec<String>>,
    #[serde(rename = "emailAddresses")]
    pub email_addresses: Option<Vec<OutlookEmailInfo>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutlookContactList {
    pub value: Vec<OutlookContactInfo>,
    #[serde(rename = "@odata.nextLink")] 
    pub next: Option<String>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct OutlookContacts {
    pub contacts: Vec<GenericContact>,
    pub next: Option<String>
}

impl<'de> Deserialize<'de> for OutlookContacts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let mut contacts: Vec<GenericContact> = Vec::new();
        let connection = OutlookContactList::deserialize(deserializer)?;
        for contact in connection.value.iter() {
            
            let mut name = String::from("");
            if let Some(names) = &contact.display_name {
                name = names.to_string();
            }

            let photo = format!("https://graph.microsoft.com/v1.0/me/contacts/{}/photo/$value", contact.id);
            
            let mut phone_numbers = Vec::new();
            if let Some(numbers) = &contact.home_phones {
                for number in numbers {
                    phone_numbers.push(number.to_string());
                }
            }
            if let Some(number) = &contact.mobile_phone {
                phone_numbers.push(number.to_string());
            }
            if let Some(numbers) = &contact.business_phones {
                for number in numbers {
                    phone_numbers.push(number.to_string());
                }
            }

            let mut email_addresses = Vec::new();
            if let Some(emails) = &contact.email_addresses {
                for email in emails {
                    email_addresses.push(email.address.as_ref().map_or(String::new(), |f| f.to_string()));
                }
            }

            let item = GenericContact {
                identifier: contact.id.clone(),
                name: Some(name),
                photo: Some(photo),
                phone_numbers: Some(phone_numbers),
                email_addresses: Some(email_addresses),
            };
            contacts.push(item);
        }
        Ok(OutlookContacts { contacts: contacts, next: connection.next })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, JsonSchema, Deserialize)]
pub struct ContactSync {
    pub email: String,
    pub phone: String,
    pub provider: Provider,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema, Deserialize)]
pub struct ContactRes {
    pub user_id: String,
    pub email: String,
    pub phone: String,
    pub provider: Provider,
    pub total: usize,
    pub contacts: Value,
}

impl Default for ContactRes {
    fn default() -> Self {
        Self {
            user_id: String::from(""),
            email: String::from(""),
            phone: String::from(""),
            provider: Provider::DefaultProvider,
            total: 0,
            contacts: serde_json::from_str("{}").unwrap(),
        }
    }
}

#[derive(FromRequest, Debug, Clone, PartialEq, Serialize, JsonSchema, Deserialize)]
#[from_request(via(Query))]
pub enum Provider {
    #[serde(rename = "google")]
    Google,
    #[serde(rename = "outlook")]
    Outlook,
    #[serde(rename = "shopify")]
    Shopify,
    #[serde(rename = "")]
    DefaultProvider,
}

impl Default for Provider {
    fn default() -> Self {
        Self::DefaultProvider
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default, Debug, Clone, PartialEq, JsonSchema, Serialize, Deserialize)]
#[query]
pub struct ContactQuery {
    pub provider: Provider,
    pub page: i64,
    pub size: i64,
    pub query: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Contact {
    #[serde(flatten)]
    pub customer: Customer,
    pub customer_orders: Vec<Order>,
    pub customer_addresses: Vec<CustomerAddress>,
}

#[derive(Clone, Debug, Deserialize, Serialize, FromRow)]
pub struct ContactModel {
    #[serde(flatten)]
    pub customer: Json<Customer>,
    pub customer_orders: Json<Vec<Order>>,
}
