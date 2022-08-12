use std::sync::Arc;
use uuid::Uuid;
use sqlx::PgPool;
use sqlx::types::chrono::NaiveDateTime;
use axum::extract::Extension;
use axum::{body::Body, extract::Query, extract::rejection::JsonRejection, response::Response, Json};
use axum_macros::debug_handler;

use crate::user::user::{
    User,
    UserQuery,
};
use crate::utils::response::{
    send_reponse,
};

// API
#[debug_handler]
pub async fn create_or_update(
    payload: Result<Json<User>, JsonRejection>,
    Extension(pool): Extension<Arc<PgPool>>
) -> Result<Response<Body>, Response<Body>> {
    match payload {
        Ok(payload) => {
            let user_info = payload.0;

            let db_user = db_update_user(&user_info, &pool).await;
            match db_user {
                Ok(_) => {
                    let encoded = serde_json::to_string(&user_info).unwrap();
                    send_reponse(200, Body::from(encoded))
                }
                Err(e) => {
                    println!("{:?}", e.to_string());
                    send_reponse(500, Body::from(format!("{:?}", e)))                     
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
pub async fn get_user(
    params: Query<UserQuery>,
    Extension(pool): Extension<Arc<PgPool>>
) -> Result<Response<Body>, Response<Body>> {
    let user = db_get_user(&params.user_id, &pool).await;
    match user {
        Ok(result) => {
            let msg_str = serde_json::to_string(&result).unwrap();
            send_reponse(200, Body::from(msg_str))
        }
        Err(e) => {
            println!("{:?}", e);
            send_reponse(500, Body::from(format!("{:?}", e)))                                    
        }
    }
}

#[debug_handler]
pub async fn delete_user(
    params: Query<UserQuery>,
    Extension(pool): Extension<Arc<PgPool>>
) -> Result<Response<Body>, Response<Body>> {
    let user = db_delete_user(&params.user_id, &pool).await;
    match user {
        Ok(_) => {
            send_reponse(200, Body::from("OK"))
        }
        Err(e) => {
            println!("{:?}", e);
            send_reponse(500, Body::from(format!("{:?}", e)))                                    
        }
    }
}

// Database
pub async fn db_update_user(user: &User, pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut invite_users = String::from("[]");
    if let Some(i) = &user.invite_users {
        invite_users = serde_json::to_string(&i).unwrap();
    }

    let mut app_ids = String::from("[]");
    if let Some(i) = &user.app_ids {
        app_ids = serde_json::to_string(&i).unwrap();
    }

    let mut post_ids = String::from("[]");
    if let Some(i) = &user.post_ids {
        post_ids = serde_json::to_string(&i).unwrap();
    }

    let mut workspace_ids = String::from("[]");
    if let Some(i) = &user.workspace_ids {
        workspace_ids = serde_json::to_string(&i).unwrap();
    }

    let mut organization = String::from("[]");
    if let Some(i) = &user.organization {
        organization = serde_json::to_string(&i).unwrap();
    }

    let mut latitude = 0.0;
    if let Some(i) = &user.latitude {
        latitude = *i;
    }

    let mut longitude = 0.0;
    if let Some(i) = &user.longitude {
        longitude = *i;
    }
    
    let _ = sqlx::query!(
        "INSERT INTO users (
            user_id, first_name, last_name, username, email, dob, last_at, two_fator, picture, gender, bio, 
            user_account_type, phone_number, invite_users, referred_by, app_ids, post_ids, workspace_ids,
            organization, latitude, longitude, last_login_ip) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
        ON CONFLICT (user_id) DO UPDATE SET first_name = $2, last_name = $3, username = $4, email = $5, dob = $6, 
        last_at = $7, two_fator = $8, picture = $9, gender = $10, bio = $11, user_account_type = $12, phone_number = $13, 
        invite_users = $14, referred_by = $15, app_ids = $16, post_ids = $17, workspace_ids = $18, organization = $19, 
        latitude = $20, longitude = $21, last_login_ip = $22",
        user.user_id,
        user.first_name,
        user.last_name,
        user.username,
        user.email,
        user.dob as NaiveDateTime,
        user.last_at as NaiveDateTime,
        user.two_fator,
        user.picture,
        user.gender,
        user.bio,
        user.user_account_type,
        user.phone_number, 
        invite_users, 
        user.referred_by, 
        app_ids, 
        post_ids, 
        workspace_ids, 
        organization, 
        latitude as f32, 
        longitude as f32, 
        user.last_login_ip,
    ).execute(pool).await?;
    Ok(())
}

pub async fn db_get_user(user_id: &String, pool: &PgPool) -> Result<User, sqlx::Error> {
    let row = sqlx::query!("SELECT * FROM users WHERE user_id = $1", user_id).fetch_one(pool).await?;
    
    let mut invite_users = Vec::new();
    if row.invite_users.is_some() {
        let _list: Vec<Uuid> = serde_json::from_str(&row.invite_users.unwrap()).unwrap();
        invite_users.extend(_list);
    }

    let mut app_ids = Vec::new();
    if row.app_ids.is_some() {
        let _list: Vec<Uuid> = serde_json::from_str(&row.app_ids.unwrap()).unwrap();
        app_ids.extend(_list);
    }

    let mut post_ids = Vec::new();
    if row.post_ids.is_some() {
        let _list: Vec<Uuid> = serde_json::from_str(&row.post_ids.unwrap()).unwrap();
        post_ids.extend(_list);
    }

    let mut workspace_ids = Vec::new();
    if row.workspace_ids.is_some() {
        let _list: Vec<Uuid> = serde_json::from_str(&row.workspace_ids.unwrap()).unwrap();
        workspace_ids.extend(_list);
    }

    let mut organization = Vec::new();
    if row.organization.is_some() {
        let _list: Vec<Uuid> = serde_json::from_str(&row.organization.unwrap()).unwrap();
        organization.extend(_list);
    }

    let mut latitude = 0.0;
    if row.latitude.is_some() {
        latitude = row.latitude.unwrap() as f64;
    }

    let mut longitude = 0.0;
    if row.longitude.is_some() {
        longitude = row.longitude.unwrap() as f64;
    }

    let mut two_fator = false;
    if row.two_fator.is_some() {
        two_fator = row.two_fator.unwrap() as bool;
    }

    let user = User {
        user_id: row.user_id,
        first_name: row.first_name,
        last_name: row.last_name,
        username: row.username,
        email: row.email,
        dob: row.dob.naive_utc(),
        last_at: row.last_at.naive_utc(),
        two_fator: Some(two_fator),
        picture: Some(row.picture).unwrap(),
        gender: Some(row.gender).unwrap(),
        bio: Some(row.bio).unwrap(),
        user_account_type: Some(row.user_account_type).unwrap(),
        phone_number: Some(row.phone_number).unwrap(), 
        invite_users: Some(invite_users), 
        referred_by: Some(row.referred_by).unwrap(), 
        app_ids: Some(app_ids), 
        post_ids: Some(post_ids), 
        workspace_ids: Some(workspace_ids), 
        organization: Some(organization),
        latitude: Some(latitude),
        longitude: Some(longitude),
        last_login_ip: Some(row.last_login_ip).unwrap(),
    };
    Ok(user)
}

pub async fn db_delete_user(user_id: &String, pool: &PgPool) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!("DELETE FROM users WHERE user_id = $1", user_id).execute(pool).await?;
    Ok(())
}