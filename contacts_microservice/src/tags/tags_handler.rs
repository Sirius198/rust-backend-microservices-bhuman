use axum::{
    extract::{rejection::JsonRejection, Query},
    Extension, Json,
};
use axum_macros::debug_handler;
use openapi_rs::openapi_proc_macro::handler;
use sqlx::PgPool;
use std::sync::Arc;
use std::fmt::Write;
use uuid::Uuid;

use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;
use openapi_rs::OpenApiFromData;

use crate::{
    contacts::param::RequiredId,
    tags::{tag::{CreateTag, UpdateTag, TagInfo}},
    groups::groups_handler::db_delete_from_tag_by_id,
};
use microservice_utils::server::response::{AxumRes,into_reponse, AxumResult};
use microservice_utils::jwt::extractor::AuthToken;

// API
#[debug_handler]
#[handler(method = "POST", tag = "tag")]
pub async fn create_tag(
    payload: Result<Json<CreateTag>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let tag_info = payload.0;
        
            let result = db_create_tag(&user_id, &tag_info, &pool)
                .await
                .map_err(|e| into_reponse(500, e.to_string().into()))
                .map(|tag| {                                  
                    axum::Json(AxumRes {
                        result: serde_json::json!(tag),
                        code: 200,
                    })
                })?;
            
            Ok(result)
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
#[handler(method = "PUT", tag = "tag")]
pub async fn update_tag(
    payload: Result<Json<UpdateTag>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let tag_info = payload.0;
        
            let result = db_update_tag(&user_id, &tag_info, &pool)
                .await
                .map_err(|e| into_reponse(500, e.to_string().into()))
                .map(|tag| {                                  
                    axum::Json(AxumRes {
                        result: serde_json::json!(tag),
                        code: 200,
                    })
                })?;
            
            Ok(result)
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
#[handler(method = "GET", tag = "tag", description = "", summary = "")]
pub async fn get_tag(
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>
) -> AxumResult<Json<AxumRes>> {
    let result = db_get_tag(&user_id, &pool)
        .await
        .map_err(|e| into_reponse(500, e.to_string().into()))
        .map(|tags| {                                  
            axum::Json(AxumRes {
                result: serde_json::json!(tags),
                code: 200,
            })
        })?;
    
    Ok(result)
}

#[debug_handler]
#[handler(method = "DELETE", tag = "tag")]
pub async fn delete_tag(
    params: Query<RequiredId>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>
) -> AxumResult<Json<AxumRes>> {
    let result = db_delete_tag(&user_id, &params, &pool)
        .await
        .map_err(|e| into_reponse(500, e.to_string().into()))
        .map(|_| {                     
            let ret = serde_json::json!({
                "status": "success",
            });               
            axum::Json(AxumRes {
                result: ret,
                code: 200,
            })
        })?;
    
    Ok(result)
}

// Database
pub async fn db_create_tag(
    user_id: &String,
    params: &CreateTag,
    pool: &PgPool,
) -> Result<TagInfo, sqlx::Error> {
    let tag = sqlx::query_as!(TagInfo, r#"INSERT INTO tag_name (user_id, name) VALUES ($1, $2) RETURNING *"#, user_id, params.name).fetch_one(pool).await?;    
    Ok(tag)
}

pub async fn db_update_tag(
    user_id: &String,
    params: &UpdateTag,
    pool: &PgPool,
) -> Result<TagInfo, sqlx::Error> {
    let tag = sqlx::query_as!(TagInfo, 
        r#"UPDATE tag_name SET name = $1 WHERE user_id = $2 AND id = $3 RETURNING *"#,
            params.name,
            user_id,
            params.id,
    ).fetch_one(pool).await?;
    Ok(tag)
}

pub async fn db_get_tag(
    user_id: &String,
    pool: &PgPool,
) -> Result<Vec<TagInfo>, sqlx::Error> {
    let mut query = "SELECT * FROM tag_name WHERE ".to_string();
    write!(query, "user_id = '{}' ", user_id).unwrap();

    let str_query: &str = &query[..];
    let tags = sqlx::query_as::<_, TagInfo>(str_query).fetch_all(pool).await?;
    Ok(tags)
}

pub async fn db_get_tag_by_id(
    user_id: &String,
    tag_id: &Uuid,
    pool: &PgPool,
) -> Result<TagInfo, sqlx::Error> {
    let tag = sqlx::query_as!(TagInfo, r#"SELECT * FROM tag_name WHERE user_id = $1 AND id = $2"#, user_id, tag_id).fetch_one(pool).await?;
    Ok(tag)  
}

pub async fn db_delete_tag(
    user_id: &String,
    params: &RequiredId,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let _ = db_delete_from_tag_by_id(&user_id, &params.id, &pool).await?;
    let _ = sqlx::query!("DELETE FROM tag_name WHERE user_id = $1 AND id = $2", user_id, params.id).execute(pool).await?;
    Ok(())
}
