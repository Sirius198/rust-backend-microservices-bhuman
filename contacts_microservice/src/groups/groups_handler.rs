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
    groups::group::{TagPeople, TagPeopleResult},
    contacts::{
        contacts::{GenericContact},
        param::RequiredId,
        contacts_handler::get_generic_contact_by_identifier,
    },
    tags::tags_handler::db_get_tag_by_id,
};
use microservice_utils::server::response::{AxumRes,into_reponse, AxumResult};
use microservice_utils::jwt::extractor::AuthToken;

// API
#[debug_handler]
#[handler(method = "POST", tag = "group")]
pub async fn add_to_tag(
    payload: Result<Json<TagPeople>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let tag_info = payload.0;
        
            let result = db_add_to_tag(&user_id, &tag_info, &pool)
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
#[handler(method = "GET", tag = "group")]
pub async fn get_from_tag(
    params: Query<RequiredId>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>
) -> AxumResult<Json<AxumRes>> {
    let result = db_get_from_tag(&user_id, &params.id, &pool)
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

#[debug_handler]
#[handler(method = "DELETE", tag = "group")]
pub async fn delete_from_tag(
    payload: Result<Json<TagPeople>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let tag_info = payload.0;
        
            let result = db_delete_from_tag(&user_id, &tag_info, &pool)
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

// Database
pub async fn db_add_to_tag(
    user_id: &String,
    params: &TagPeople,
    pool: &PgPool,
) -> Result<TagPeopleResult, sqlx::Error> {
    // Check contact identifier
    let _ = get_generic_contact_by_identifier(&user_id, &params.identifier, &pool).await?;

    // Check tag
    let _ = db_get_tag_by_id(&user_id, &params.tag_id, &pool).await?;

    let _ = sqlx::query!("INSERT INTO tag_contacts (user_id, tag_id, identifier) VALUES ($1, $2, $3) RETURNING *", 
        user_id, params.tag_id, params.identifier).fetch_one(pool).await?;  
        
    Ok(db_get_from_tag(&user_id, &params.tag_id, &pool).await?)
}

pub async fn db_get_from_tag(
    user_id: &String,
    tag_id: &Uuid,
    pool: &PgPool,
) -> Result<TagPeopleResult, sqlx::Error> {
    let mut query = "SELECT jt.* FROM (SELECT tag_id, ARRAY_AGG(DISTINCT identifier) AS ids FROM tag_contacts WHERE ".to_string();
    write!(query, "user_id = '{}' ", user_id).unwrap();
    write!(query, "AND tag_id = '{}' ", tag_id).unwrap();
    write!(query, "GROUP BY tag_id) q LEFT JOIN generic_contacts jt ON jt.identifier = ANY(q.ids)").unwrap();

    let str_query: &str = &query[..];
    let contacts = sqlx::query_as::<_, GenericContact>(str_query).fetch_all(pool).await?;

    let res = TagPeopleResult {
        user_id: user_id.to_string(),
        tag_id: *tag_id,
        contacts: contacts,
    };
    Ok(res)
}

pub async fn db_delete_from_tag(
    user_id: &String,
    params: &TagPeople,
    pool: &PgPool,
) -> Result<TagPeopleResult, sqlx::Error> {
    // Check contact identifier
    let _ = get_generic_contact_by_identifier(&user_id, &params.identifier, &pool).await?;

    // Check tag
    let _ = db_get_tag_by_id(&user_id, &params.tag_id, &pool).await?;

    let _ = sqlx::query!("DELETE FROM tag_contacts WHERE user_id = $1 AND tag_id = $2 AND identifier = $3", 
        user_id, params.tag_id, params.identifier).execute(pool).await?; 
        
    Ok(db_get_from_tag(&user_id, &params.tag_id, &pool).await?)
}

pub async fn db_delete_from_tag_by_id(
    user_id: &String,
    tag_id: &Uuid,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!("DELETE FROM tag_contacts WHERE user_id = $1 AND tag_id = $2", 
        user_id, tag_id).execute(pool).await?;        
    Ok(())
}