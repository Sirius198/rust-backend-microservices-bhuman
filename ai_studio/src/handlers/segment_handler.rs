use axum::{
    extract::{rejection::JsonRejection, Extension, Query},
    Json,
};
use axum_macros::debug_handler;
use sqlx::PgPool;
use std::fmt::Write;
use std::sync::Arc;
use chrono::Utc;
use chrono::NaiveDateTime;

use openapi_rs::openapi_proc_macro::handler;
use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;
use openapi_rs::OpenApiFromData;

use crate::models::segment::{CreateSegment, Segment, SegmentOptionalId, UpdateSegment};
use microservice_utils::{jwt::extractor::AuthToken, server::response::{into_reponse, AxumResult, AxumRes}};

// API
#[debug_handler]
#[handler(method = "POST",tag = "segment")]
pub async fn create_segment(
    payload: Result<Json<CreateSegment>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let segment_info = payload.0;

            match db_create_segment(&user_id, &segment_info, &pool).await {
                Ok(result) => Ok(axum::Json(AxumRes{code:200, result:serde_json::json!(result)})),
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
#[handler(method = "PUT",tag = "segment")]
pub async fn update_segment(
    payload: Result<Json<UpdateSegment>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let segment_info = payload.0;

            match db_update_segment(&user_id, &segment_info, &pool).await {
                Ok(result) => Ok(axum::Json(AxumRes{code:200, result:serde_json::json!(result)})),
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
#[handler(method = "GET",tag = "segment")]
pub async fn get_segment(
    params: Query<SegmentOptionalId>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    let segments = db_get_segment(&user_id, &params, &pool).await;
    match segments {
        Ok(result) => Ok(axum::Json(AxumRes{code:200, result:serde_json::json!(result)})),
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
#[handler(method = "DELETE",tag = "segment")]
pub async fn delete_segment(
    payload: Result<Json<SegmentOptionalId>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let segment_info = payload.0;

            let users = db_delete_segment(&user_id, &segment_info, &pool).await;
            match users {
                Ok(_) => {
                    let ret = serde_json::json!({
                        "status": "success",
                    });
                    Ok(axum::Json(AxumRes{code:200, result:ret}))
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
pub async fn db_create_segment(
    user_id: &String,
    segment: &CreateSegment,
    pool: &PgPool,
) -> Result<Segment, sqlx::Error> {
    let _ = sqlx::query!(
        "SELECT id FROM video_instances WHERE id=$1 AND user_id = $2",
        segment.video_instance_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    let out_segment = sqlx::query_as!(Segment, 
        r#"INSERT INTO segments (
            user_id ,video_instance_id, prefix_time_marker_start, prefix_time_marker_end,
            suffix_time_marker_start, suffix_time_marker_end, audio_variable_column_id, audio_variable_name,
            variable_time_marker_start, variable_time_marker_end) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *"#,
            user_id,
            segment.video_instance_id,
            segment.prefix_time_marker_start,
            segment.prefix_time_marker_end,
            segment.suffix_time_marker_start,
            segment.suffix_time_marker_end,
            segment.audio_variable_column_id,
            segment.audio_variable_name.to_lowercase(),
            segment.variable_time_marker_start,
            segment.variable_time_marker_end,
    ).fetch_one(pool).await?;   
    Ok(out_segment)
}

pub async fn db_update_segment(
    user_id: &String,
    params: &UpdateSegment,
    pool: &PgPool,
) -> Result<Segment, sqlx::Error> {
    let out_segment = sqlx::query_as!(Segment, 
            r#"UPDATE segments SET audio_variable_name = $1, updated_at = $2 WHERE id = $3 AND user_id = $4 RETURNING *"#,
            params.audio_variable_name.to_lowercase(),
            Utc::now().naive_utc() as NaiveDateTime,
            params.id,
            user_id
        )
        .fetch_one(pool)
        .await?;    
    Ok(out_segment)
}

pub async fn db_get_segment(
    user_id: &String,
    params: &SegmentOptionalId,
    pool: &PgPool,
) -> Result<Vec<Segment>, sqlx::Error> {
    let mut query = "SELECT * FROM segments WHERE ".to_string();
    write!(query, "user_id = '{}' ", user_id).unwrap();

    if let Some(params_id) = params.id {
        write!(query, "And id = '{}' ", params_id).unwrap();
    }
    if let Some(params_video_instance_id) = params.video_instance_id {
        write!(
            query,
            "And video_instance_id = '{}' ",
            params_video_instance_id
        )
        .unwrap();
    }

    let str_query: &str = &query[..];
    let out_segments = sqlx::query_as::<_, Segment>(str_query)
        .fetch_all(pool)
        .await?;
    Ok(out_segments)
}

pub async fn db_delete_segment(
    user_id: &String,
    params: &SegmentOptionalId,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let mut query = "DELETE FROM segments WHERE ".to_string();
    write!(query, "user_id = '{}' ", user_id).unwrap();
    
    if let Some(params_id) = params.id {
        write!(query, "And id = '{}' ", params_id).unwrap();
    }
    if let Some(params_video_instance_id) = params.video_instance_id{
        write!(query, "And video_instance_id = '{}' ", params_video_instance_id).unwrap();
    }

    let str_query: &str = &query[..];
    let _ = sqlx::query(str_query).execute(pool).await?;
    Ok(())
}
