use sqlx::PgPool;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use axum::{extract::Extension, extract::Query, body::Body, extract::rejection::JsonRejection, response::Response, Json};
use axum_macros::debug_handler;
use headers::{authorization::Bearer, Authorization};

use crate::contacts::contacts::{
    ContactSync,
    ContactList,
    ContactRes,
    ContactQuery,
};

use crate::utils::{
    response::send_reponse,
};

#[debug_handler]
pub async fn sync_contacts(
    payload: Result<Json<ContactSync>, JsonRejection>,
    Extension(pool): Extension<Arc<PgPool>>
) -> Result<Response<Body>, Response<Body>> {
    match payload {
        Ok(payload) => {
            let sync_info = payload.0;
            println!("Received token {:?}", sync_info);

            let client = reqwest::Client::new();

            if sync_info.provider == "google" {
                let person_fields = String::from("addresses,birthdays,emailAddresses,genders,names,organizations,phoneNumbers,photos,userDefined");                
                let request = client
                    .get(format!("https://people.googleapis.com/v1/people/me/connections?personFields={}", person_fields))
                    .header("Content-Type", "application/json")
                    .header(reqwest::header::USER_AGENT, "curl/7.64.1")
                    .header("Authorization", format!("Bearer {}", sync_info.token))
                    .send()
                    .await
                    .unwrap();
    
                let response = request.text().await.unwrap();
    
                let mut contacts: ContactList = serde_json::from_str(&response).unwrap();
                contacts.e164_format();
    
                let encoded_contacts = serde_json::to_string(&contacts).unwrap();
                let add_contacts = sync_google_contacts(
                    &sync_info.user_id, 
                    &sync_info.phone,
                    &sync_info.email, 
                    &encoded_contacts,
                    &pool
                ).await;
    
                match add_contacts {
                    Ok(_) => {    
                        let response: ContactRes = ContactRes {
                            user_id: sync_info.user_id,
                            phone: sync_info.phone,
                            email: sync_info.email,
                            provider: sync_info.provider,
                            contacts: encoded_contacts,
                        };
                        let encoded_contacts = serde_json::to_string(&response).unwrap();
                        send_reponse(200, Body::from(encoded_contacts))
                    }
                    Err(e) => {
                        println!("{:?}", e.to_string());
                        send_reponse(500, Body::from(format!("{:?}", e)))                     
                    }
                }
            } else {
                let person_fields = String::from("givenName,surname,emailAddresses,mobilePhone");                
                let request = client
                    .get(format!("https://graph.microsoft.com/v1.0/me/contacts?personFields={}", person_fields))
                    .header("Content-Type", "application/json")
                    .header(reqwest::header::USER_AGENT, "curl/7.64.1")
                    .header("Authorization", format!("Bearer {}", sync_info.token))
                    .send()
                    .await
                    .unwrap();
    
                let response = request.text().await.unwrap();

                let mut contacts: ContactList = ContactList::from(response);
                contacts.e164_format();
                
                let encoded_contacts = serde_json::to_string(&contacts).unwrap();
                let add_contacts = sync_outlook_contacts(
                    &sync_info.user_id, 
                    &sync_info.phone,
                    &sync_info.email, 
                    &encoded_contacts,
                    &pool
                ).await;
    
                match add_contacts {
                    Ok(_) => {    
                        let response: ContactRes = ContactRes {
                            user_id: sync_info.user_id,
                            phone: sync_info.phone,
                            email: sync_info.email,
                            provider: sync_info.provider,
                            contacts: encoded_contacts,
                        };
                        let encoded_contacts = serde_json::to_string(&response).unwrap();
                        send_reponse(200, Body::from(encoded_contacts))
                    }
                    Err(e) => {
                        println!("{:?}", e.to_string());
                        send_reponse(500, Body::from(format!("{:?}", e)))                     
                    }
                }
            }            
        }
        Err(e) => {
            println!("{:?}", e.to_string());
            send_reponse(400, Body::from(format!("{:?}", e)))
        }
    }
}

#[debug_handler]
pub async fn get_contacts(
    params: Query<ContactQuery>,
    Extension(pool): Extension<Arc<PgPool>>
) -> Result<Response<Body>, Response<Body>> {
    if params.provider == "google" {
        let contacts = get_google_contacts(&params.user_id, &pool).await;        
        match contacts {
            Ok(result) => {
                println!("Fetch google contacts");
                let msg_str = serde_json::to_string(&result).unwrap();
                send_reponse(200, Body::from(msg_str))
            }
            Err(e) => {
                println!("{:?}", e);
                send_reponse(500, Body::from(format!("{:?}", e)))                                    
            }
        }
    } else {
        let contacts = get_outlook_contacts(&params.user_id, &pool).await;
        match contacts {
            Ok(result) => {
                println!("Fetch outlook contacts");
                let msg_str = serde_json::to_string(&result).unwrap();
                send_reponse(200, Body::from(msg_str))
            }
            Err(e) => {
                println!("{:?}", e);
                send_reponse(500, Body::from(format!("{:?}", e)))                                    
            }
        }
    }
}

// Database
pub async fn sync_google_contacts(
    user_id: &String,
    phone: &String,
    email: &String,
    contacts: &String,
    pool: &PgPool
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        "INSERT INTO contacts (user_id, phone, email, google_contacts) VALUES ($1, $2, $3, $4) ON CONFLICT (user_id) DO UPDATE SET phone = $2, email = $3, google_contacts = $4",
        user_id,
        phone,
        email,
	    contacts,
    ).execute(pool).await?;
    Ok(())
}

pub async fn sync_outlook_contacts(
    user_id: &String,
    phone: &String,
    email: &String,
    contacts: &String,
    pool: &PgPool
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        "INSERT INTO contacts (user_id, phone, email, outlook_contacts) VALUES ($1, $2, $3, $4) ON CONFLICT (user_id) DO UPDATE SET phone = $2, email = $3, outlook_contacts = $4",
        user_id,
        phone,
        email,
	    contacts,
    ).execute(pool).await?;
    Ok(())
}

pub async fn get_google_contacts(
    user_id: &String, 
    pool: &PgPool
) -> Result<ContactRes, sqlx::Error> {
    let row = sqlx::query!("SELECT user_id, phone, email, google_contacts FROM contacts WHERE user_id = $1", user_id).fetch_one(pool).await?;
    let mut phone = String::from("");
    if row.phone.is_some() {
        phone = row.phone.unwrap();
    }
    let mut email = String::from("");
    if row.email.is_some() {
        email = row.email.unwrap();
    }
    let mut google_contacts = String::from("");
    if row.google_contacts.is_some() {
        google_contacts = row.google_contacts.unwrap();
    }
    let res = ContactRes {
        user_id: row.user_id,
        phone: phone,
        email: email,
        provider: "google".to_string(),
        contacts: google_contacts,
    };
    Ok(res)
}

pub async fn get_outlook_contacts(
    user_id: &String, 
    pool: &PgPool
) -> Result<ContactRes, sqlx::Error> {
    let row = sqlx::query!("SELECT user_id, phone, email, outlook_contacts FROM contacts WHERE user_id = $1", user_id).fetch_one(pool).await?;
    let mut phone = String::from("");
    if row.phone.is_some() {
        phone = row.phone.unwrap();
    }
    let mut email = String::from("");
    if row.email.is_some() {
        email = row.email.unwrap();
    }
    let mut outlook_contacts = String::from("");
    if row.outlook_contacts.is_some() {
        outlook_contacts = row.outlook_contacts.unwrap();
    }
    let res = ContactRes {
        user_id: row.user_id,
        phone: phone,
        email: email,
        provider: "outlook".to_string(),
        contacts: outlook_contacts,
    };
    Ok(res)
}