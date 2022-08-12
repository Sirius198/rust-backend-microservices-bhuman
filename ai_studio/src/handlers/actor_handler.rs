use axum::{
    extract::{rejection::JsonRejection, Extension, Query},
    Json,
};
use axum_macros::debug_handler;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use std::fmt::Write;
use chrono::Utc;
use chrono::NaiveDateTime;

use openapi_rs::openapi_proc_macro::handler;
use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;
use openapi_rs::OpenApiFromData;

use crate::models::actor::{Actor, CreateActor, UpdateActor};
use crate::models::param::{OptionalId, RequiredId};
use microservice_utils::{jwt::extractor::AuthToken, server::response::{into_reponse, AxumResult, AxumRes}};

// API
#[debug_handler]
#[handler(method = "POST",tag = "actor")]
pub async fn create_actor(
    payload: Result<Json<CreateActor>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let actor_info = payload.0;

            match db_create_actor(&user_id, &actor_info, &pool).await {
                Ok(result) => Ok(axum::Json(AxumRes {code: 200, result: serde_json::json!(&result)})),
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
#[handler(method = "PUT",tag = "actor")]
pub async fn update_actor(
    payload: Result<Json<UpdateActor>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let actor_info = payload.0;

            match db_update_actor(&user_id, &actor_info, &pool).await {
                Ok(result) => Ok(axum::Json(AxumRes {code: 200, result: serde_json::json!(&result)})),
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
#[handler(method = "GET",tag = "actor")]
pub async fn get_actor(
    params: Query<OptionalId>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    let actors = db_get_actor(&user_id, &params, &pool).await;
    match actors {
        Ok(result) => Ok(axum::Json(AxumRes {code: 200, result: serde_json::json!(&result)})),
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
#[handler(method = "DELETE",tag = "actor")]
pub async fn delete_actor(
    payload: Result<Json<RequiredId>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let actor_info = payload.0;

            let res = db_delete_actor(&user_id, &actor_info, &pool).await;
            match res {
                Ok(_) => {
                    let ret = serde_json::json!({
                        "status": "success",
                    });
                    Ok(axum::Json(AxumRes {code: 200, result: serde_json::json!(&ret)}))
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

// Database
pub async fn db_create_actor(
    user_id: &String,
    actor: &CreateActor,
    pool: &PgPool,
) -> Result<Actor, sqlx::Error> {
    let out_actor = sqlx::query_as!(
        Actor,
        r#"INSERT INTO actors (
            user_id, name) VALUES ($1, $2) RETURNING *"#,
        user_id,
        actor.name,
    )
    .fetch_one(pool)
    .await?;
    Ok(out_actor)
}

pub async fn db_update_actor(
    user_id: &String,
    actor: &UpdateActor,
    pool: &PgPool,
) -> Result<Actor, sqlx::Error> {
    let out_actor = sqlx::query_as!(Actor, 
        r#"UPDATE actors SET name = $1, updated_at = $2 WHERE id = $3 AND user_id = $4 RETURNING *"#,
        actor.name,
        Utc::now().naive_utc() as NaiveDateTime,
        actor.id,
        user_id
    )
    .fetch_one(pool)
    .await?;    
    Ok(out_actor)
}

pub async fn db_get_actor(
    user_id: &String,
    params: &OptionalId,
    pool: &PgPool,
) -> Result<Vec<Actor>, sqlx::Error> {
    let mut query = "SELECT * FROM actors WHERE ".to_string();
    write!(query, "user_id = '{}' ", user_id).unwrap();

    if let Some(params_id) = params.id {
        write!(query, "And id = '{}' ", params_id).unwrap();
    }

    let str_query: &str = &query[..];
    let actors = sqlx::query_as::<_, Actor>(str_query)
        .fetch_all(pool)
        .await?;
    Ok(actors)
}

pub async fn db_delete_actor(
    user_id: &String,
    params: &RequiredId,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    match db_check_audios(&user_id, &params.id, &pool).await {
        Ok(_) => Err(sqlx::Error::Protocol(
            "Can't delete, this actor have audio".to_string(),
        )),
        Err(_) => {
            let _ = sqlx::query!(
                "DELETE FROM actors WHERE id = $1 AND user_id = $2",
                params.id,
                user_id
            )
            .execute(pool)
            .await?;
            Ok(())
        }
    }
}

pub async fn db_check_audios(
    user_id: &String,
    actor_id: &Uuid,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        "SELECT id FROM audios WHERE actor_id = $1 AND user_id = $2",
        actor_id,
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(())
}
