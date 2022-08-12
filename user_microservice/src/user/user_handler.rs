use std::sync::Arc;
use openapi_rs::openapi_proc_macro::handler;
use uuid::Uuid;
use sqlx::PgPool;
use sqlx::types::chrono::NaiveDateTime;
use axum::extract::Extension;
use axum::{extract::{rejection::JsonRejection}, Json};
use axum_macros::debug_handler;
use tonic::async_trait;
use sqlx::types::chrono::Utc;
use std::fmt::Write;

use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;
use openapi_rs::OpenApiFromData;

use crate::user_service::user_service_server::UserService;
use crate::user_service::{AddWorkspaceRequest, AddWorkspaceResponse, RemoveWorkspaceRequest, RemoveWorkspaceResponse};

use crate::user::user::{
    CreateUser,
    UpdateUser,
    User,
};
use microservice_utils::{jwt::{extractor::AuthToken}, server::response::{into_reponse,AxumResult,AxumRes}};

// gRPC
pub struct MyUserService {
    pool: PgPool
}

impl MyUserService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

#[async_trait]
impl UserService for MyUserService {
    async fn add_workspace_id(
        &self,
        request: tonic::Request<AddWorkspaceRequest>,
    ) -> Result<tonic::Response<AddWorkspaceResponse>, tonic::Status> {

        let req: AddWorkspaceRequest = request.into_inner();
        println!("Adding workspace {:?}", req);

        let _ = add_workspace_id(&req.user_id, &req.workspace_id, &self.pool).await;

        Ok(tonic::Response::new(AddWorkspaceResponse {
            status: "Ok".to_string(),
        }))
    }

    async fn remove_workspace_id(
        &self,
        request: tonic::Request<RemoveWorkspaceRequest>,
    ) -> Result<tonic::Response<RemoveWorkspaceResponse>, tonic::Status> {

        let req: RemoveWorkspaceRequest = request.into_inner();
        println!("Removing workspace {:?}", req);

        let _ = remove_workspace_id(&req.user_id, &req.workspace_id, &self.pool).await;

        Ok(tonic::Response::new(RemoveWorkspaceResponse {
            status: "Ok".to_string(),
        }))
    }
}

// API
#[debug_handler]
#[handler(method = "POST",tag = "user")]
pub async fn create_user(
    payload: Result<Json<CreateUser>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let user_info = payload.0;

            let db_user = db_create_user(&user_id, &user_info, &pool).await;
            match db_user {
                Ok(result) => {
                    Ok(axum::Json(AxumRes{code:200, result: serde_json::json!(&result)}))
                }
                Err(e) => {
                    println!("{:?}", e.to_string());
                    let ret = serde_json::json!({
                        "error": format!("{:?}", e),
                    });    
                    Err(into_reponse(500, ret))
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
#[handler(method = "PUT",tag = "user")]
pub async fn update_user(
    payload: Result<Json<UpdateUser>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let user_info = payload.0;

            let db_user = db_update_user(&user_id, &user_info, &pool).await;
            match db_user {
                Ok(result) => {
                    Ok(axum::Json(AxumRes{code:200, result: serde_json::json!(&result)}))
                }
                Err(e) => {
                    println!("{:?}", e.to_string());
                    let ret = serde_json::json!({
                        "error": format!("{:?}", e),
                    });    
                    Err(into_reponse(500, ret))                     
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
#[handler(method = "GET",tag = "user")]
pub async fn get_user(
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>
) -> AxumResult<Json<AxumRes>> {
    let user = db_get_user(&user_id, &pool).await;
    match user {
        Ok(result) => {
            Ok(axum::Json(AxumRes{code:200, result: serde_json::json!(&result)}))
        }
        Err(e) => {
            println!("{:?}", e.to_string());
            let ret = serde_json::json!({
                "error": format!("{:?}", e),
            });    
            Err(into_reponse(500, ret))                                   
        }
    }
}

#[debug_handler]
#[handler(method = "DELETE",tag = "user")]
pub async fn delete_user(
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>
) -> AxumResult<Json<AxumRes>> {
    let user = db_delete_user(&user_id, &pool).await;
    match user {
        Ok(_) => {
            let ret = serde_json::json!({
                "status": "success",
            });  
            Ok(axum::Json(AxumRes{code:200, result: serde_json::json!(&ret)}))
        }
        Err(e) => {
            println!("{:?}", e.to_string());
            let ret = serde_json::json!({
                "error": format!("{:?}", e),
            });    
            Err(into_reponse(500, ret))
        }
    }
}

// Database
pub async fn db_create_user(user_id: &String, user: &CreateUser, pool: &PgPool) -> Result<User, sqlx::Error> {

    let mut email = String::from("");
    if let Some(_email) = &user.email {
        email = _email.to_string();
    }

    let mut phone_number = String::from("");
    if let Some(_phone_number) = &user.phone_number {
        phone_number = _phone_number.to_string();
    }

    let user = sqlx::query_as!(User,
        "INSERT INTO users (user_id, first_name, last_name, email, phone_number) 
        VALUES ($1, $2, $3, $4, $5) ON CONFLICT (user_id) DO UPDATE SET first_name = $2, last_name = $3, email = $4, phone_number = $5 RETURNING *",
        user_id,
        user.first_name,
        user.last_name,
        email,
        phone_number,
    ).fetch_one(pool).await?;    
    Ok(user)
}

pub async fn db_update_user(user_id: &String, user: &UpdateUser, pool: &PgPool) -> Result<User, sqlx::Error> {
    let mut query = "UPDATE users SET ".to_string();
    write!(query, "last_at = '{:?}' ,", Utc::now().naive_utc() as NaiveDateTime).unwrap();

    if let Some(first_name) = &user.first_name {
        write!(query, "first_name = '{}' ,", first_name).unwrap();
    }

    if let Some(last_name) = &user.last_name {
        write!(query, "last_name = '{}' ,", last_name).unwrap();
    }

    if let Some(username) = &user.username {
        write!(query, "username = '{}' ,", username).unwrap();
    }

    if let Some(email) = &user.email {
        write!(query, "email = '{}' ,", email).unwrap();
    }

    if let Some(dob) = &user.dob {
        write!(query, "dob = '{}' ,", dob).unwrap();
    }

    if let Some(two_fator) = &user.two_fator {
        write!(query, "two_fator = '{}' ,", two_fator).unwrap();
    }

    if let Some(picture) = &user.picture {
        write!(query, "picture = '{}' ,", picture).unwrap();
    }

    if let Some(gender) = &user.gender {
        write!(query, "gender = '{}' ,", gender).unwrap();
    }

    if let Some(bio) = &user.bio {
        write!(query, "bio = '{}' ,", bio).unwrap();
    }

    if let Some(user_account_type) = &user.user_account_type {
        write!(query, "user_account_type = '{}' ,", user_account_type).unwrap();
    }

    if let Some(phone_number) = &user.phone_number {
        write!(query, "phone_number = '{}' ,", phone_number).unwrap();
    }

    if let Some(latitude) = &user.latitude {
        write!(query, "latitude = '{}' ,", latitude).unwrap();
    }

    if let Some(longitude) = &user.longitude {
        write!(query, "longitude = '{}' ,", longitude).unwrap();
    }

    if let Some(last_login_ip) = &user.last_login_ip {
        write!(query, "last_login_ip = '{}' ,", last_login_ip).unwrap();
    }

    query.pop();

    write!(query, "WHERE user_id = '{}' RETURNING *", user_id).unwrap();

    let str_query: &str = &query[..];
    let out_user = sqlx::query_as::<_, User>(str_query)
        .fetch_one(pool)
        .await?;
    Ok(out_user)    
}

pub async fn db_get_user(user_id: &String, pool: &PgPool) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as!(User, r#"SELECT * FROM users WHERE user_id = $1"#, user_id).fetch_one(pool).await?;   
    Ok(user)
}

pub async fn db_delete_user(user_id: &String, pool: &PgPool) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!("DELETE FROM users WHERE user_id = $1", user_id).execute(pool).await?;
    Ok(())
}

pub async fn add_workspace_id(user_id: &String, workspace_id: &String, pool: &PgPool) -> Result<(), sqlx::Error> {
    let row = sqlx::query!("SELECT workspace_ids FROM users WHERE user_id = $1", user_id).fetch_one(pool).await?;
    let mut workspace_ids: Vec<Uuid> = Vec::new();
    if row.workspace_ids.is_some() {
        workspace_ids.extend(&row.workspace_ids.unwrap());
    }

    if workspace_ids.iter().any(|&ws| ws.to_string() == *workspace_id) == false {
        workspace_ids.push(Uuid::parse_str(workspace_id).unwrap());
    }
    
    let _ = sqlx::query!("UPDATE users SET workspace_ids = $1 WHERE user_id = $2", &workspace_ids, user_id).execute(pool).await?;
    Ok(())
}

pub async fn remove_workspace_id(user_id: &String, workspace_id: &String, pool: &PgPool) -> Result<(), sqlx::Error> {
    let row = sqlx::query!("SELECT workspace_ids FROM users WHERE user_id = $1", user_id).fetch_one(pool).await?;
    let mut workspace_ids: Vec<Uuid> = Vec::new();
    if row.workspace_ids.is_some() {
        workspace_ids.extend(&row.workspace_ids.unwrap());
    }

    let index = workspace_ids.iter().position(|ws| ws.to_string() == *workspace_id).unwrap();
    if workspace_ids.get(index).is_none() == false {
        workspace_ids.swap_remove(index);
    }

    let _ = sqlx::query!("UPDATE users SET workspace_ids = $1 WHERE user_id = $2", &workspace_ids, user_id).execute(pool).await?;
    Ok(())
}