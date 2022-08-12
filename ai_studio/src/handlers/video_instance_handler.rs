use axum::{
    extract::{rejection::JsonRejection, Extension, Query},
    Json,
};
use axum_macros::debug_handler;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::types::chrono::Utc;
use sqlx::PgPool;
use std::fmt::Write;
use std::sync::Arc;
use uuid::Uuid;

use openapi_rs::openapi_proc_macro::handler;
use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;
use openapi_rs::OpenApiFromData;

use crate::handlers::actor_handler::db_get_actor;
use crate::models::audio::AudioBatch;
use crate::models::param::{OptionalId, RequiredId};
use crate::models::video::{
    CreateVideoInstance, GeneratedVideo, UpdateVideoinstance, Video, VideoInstance,
};
use microservice_utils::{
    jwt::extractor::AuthToken,
    server::response::{into_reponse, AxumRes, AxumResult},
};

// API
#[debug_handler]
#[handler(method = "POST",tag = "video_instance")]
pub async fn create_video_instance(
    payload: Result<Json<CreateVideoInstance>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let inst_info = payload.0;

            match db_create_v_instance(&user_id, &inst_info, &pool).await {
                Ok(result) => Ok(axum::Json(AxumRes {
                    code: 200,
                    result: serde_json::json!(&result),
                })),
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
#[handler(method = "PUT",tag = "video_instance")]
pub async fn update_video_instance(
    payload: Result<Json<UpdateVideoinstance>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let inst_info = payload.0;

            match db_update_v_instance(&user_id, &inst_info, &pool).await {
                Ok(result) => Ok(axum::Json(AxumRes {
                    code: 200,
                    result: serde_json::json!(&result),
                })),
                Err(e) => {
                    println!("{:?}", e.to_string());
                    let ret = serde_json::json!({
                        "error": format!("{:?}", e),
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
#[handler(method = "GET",tag = "video_instance")]
pub async fn get_video_instance(
    params: Query<OptionalId>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    let instances = db_get_v_instance(&user_id, &params, &pool).await;
    match instances {
        Ok(result) => Ok(axum::Json(AxumRes {
            code: 200,
            result: serde_json::json!(&result),
        })),
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
#[handler(method = "DELETE",tag = "video_instance")]
pub async fn delete_video_instance(
    payload: Result<Json<RequiredId>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let inst_info = payload.0;

            let res = db_delete_v_instance(&user_id, &inst_info, &pool).await;
            match res {
                Ok(_) => {
                    let ret = serde_json::json!({
                        "status": "success",
                    });
                    Ok(axum::Json(AxumRes {
                        code: 200,
                        result: ret,
                    }))
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
pub async fn db_create_v_instance(
    user_id: &String,
    v_inst: &CreateVideoInstance,
    pool: &PgPool,
) -> Result<VideoInstance, sqlx::Error> {
    let _ = sqlx::query!(
        "SELECT id FROM folders WHERE id=$1 AND user_id = $2",
        v_inst.folder_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    let out_inst = sqlx::query_as!(
        VideoInstance,
        r#"INSERT INTO video_instances (
            name, user_id, folder_id) VALUES ($1, $2, $3) RETURNING *"#,
        v_inst.name,
        user_id,
        v_inst.folder_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(out_inst)
}

pub async fn db_update_v_instance(
    user_id: &String,
    v_inst: &UpdateVideoinstance,
    pool: &PgPool,
) -> Result<VideoInstance, sqlx::Error> {
    let mut video_id_validation: bool = true;
    let mut actor_id_validation: bool = true;
    let mut audio_batch_id_validation: bool = true;

    let mut query = "UPDATE video_instances SET ".to_string();
    write!(
        query,
        "updated_at = '{}' ,",
        Utc::now().naive_utc() as NaiveDateTime
    )
    .unwrap();

    if let Some(video_name) = &v_inst.name {
        write!(query, "name = '{}' ,", video_name).unwrap();
    }

    if let Some(video_id) = v_inst.video_id {
        let res = db_get_video(&user_id, &video_id, &pool).await;
        if res.is_ok() {
            write!(query, "video_id = '{}' ,", video_id).unwrap();
            video_id_validation = true;
        } else {
            video_id_validation = false;
        }
    }

    if let Some(column_id) = v_inst.image_column_id {
        write!(query, "image_column_id = '{}' ,", column_id).unwrap();
    }

    if let Some(actor_id) = v_inst.actor_id {
        let actor = OptionalId { id: Some(actor_id) };
        let res = db_get_actor(&user_id, &actor, &pool).await;
        if res.is_ok() {
            write!(query, "actor_id = '{}' ,", actor_id).unwrap();
            actor_id_validation = true;
        } else {
            actor_id_validation = false;
        }
    }

    if let Some(audio_batch_id) = v_inst.audio_batch_id {
        let res = db_get_audio_batch(&user_id, &audio_batch_id, &pool).await;
        if res.is_ok() {
            write!(query, "audio_batch_id = '{}' ,", audio_batch_id).unwrap();
            audio_batch_id_validation = true;
        } else {
            audio_batch_id_validation = false;
        }
    }

    query.pop();

    if !video_id_validation {
        Err(sqlx::Error::Protocol(
            "Video_id not exists for this user".to_string(),
        ))
    } else if !actor_id_validation {
        Err(sqlx::Error::Protocol(
            "Actor_id not exists for this user".to_string(),
        ))
    } else if !audio_batch_id_validation {
        Err(sqlx::Error::Protocol(
            "Audio_batch_id not exists for this user".to_string(),
        ))
    } else {
        write!(query, "WHERE id = '{}' ", v_inst.id).unwrap();
        write!(query, "And user_id = '{}' RETURNING *", user_id).unwrap();

        let str_query: &str = &query[..];
        let out_inst = sqlx::query_as::<_, VideoInstance>(str_query)
            .fetch_one(pool)
            .await?;
        Ok(out_inst)
    }
}

pub async fn db_get_video(
    user_id: &String,
    id: &Uuid,
    pool: &PgPool,
) -> Result<Video, sqlx::Error> {
    let out_video = sqlx::query_as!(
        Video,
        r#"SELECT * FROM videos WHERE id = $1 AND user_id = $2"#,
        id,
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(out_video)
}

pub async fn db_get_audio_batch(
    user_id: &String,
    id: &Uuid,
    pool: &PgPool,
) -> Result<AudioBatch, sqlx::Error> {
    let out_audio_batch = sqlx::query_as!(
        AudioBatch,
        r#"SELECT * FROM audio_batch WHERE id = $1 AND user_id = $2"#,
        id,
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(out_audio_batch)
}

pub async fn db_get_v_instance(
    user_id: &String,
    params: &OptionalId,
    pool: &PgPool,
) -> Result<Vec<VideoInstance>, sqlx::Error> {
    let mut query = "SELECT * FROM video_instances WHERE ".to_string();
    write!(query, "user_id = '{}' ", user_id).unwrap();

    if let Some(params_id) = params.id {
        write!(query, "And id = '{}' ", params_id).unwrap();
    }

    let str_query: &str = &query[..];
    let instances = sqlx::query_as::<_, VideoInstance>(str_query)
        .fetch_all(pool)
        .await?;
    Ok(instances)
}

pub async fn db_delete_v_instance(
    user_id: &String,
    params: &RequiredId,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    match db_check_generated_videos(&user_id, &params, &pool).await {
        Ok(_) => Err(sqlx::Error::Protocol(
            "Can't delete, this Video Instance have generated videos".to_string(),
        )),
        Err(_) => {
            let _ = sqlx::query!(
                "DELETE FROM video_instances WHERE id = $1 AND user_id = $2",
                params.id,
                user_id
            )
            .execute(pool)
            .await?;
            Ok(())
        }
    }
}

pub async fn db_check_generated_videos(
    user_id: &String,
    params: &RequiredId,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        "SELECT id FROM generated_videos WHERE video_instance_id = $1 AND user_id = $2",
        params.id,
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(())
}

pub async fn db_get_generated_videos(
    user_id: &String,
    params: &RequiredId,
    pool: &PgPool,
) -> Result<Vec<GeneratedVideo>, sqlx::Error> {
    let mut sql = "SELECT * FROM generated_videos WHERE ".to_string();
    write!(
        sql,
        "user_id = '{}' AND video_instance_id = '{}'",
        user_id, params.id
    )
    .unwrap();

    let str_query: &str = &sql[..];
    let res = sqlx::query_as::<_, GeneratedVideo>(str_query)
        .fetch_all(pool)
        .await?;
    Ok(res)
}

pub async fn create_batch_data(
    payload: Vec<Vec<String>>,
    audio_batch_id: &Uuid,
    user_id: &String,
    pool: &PgPool,
) {
    let mut query =
        "INSERT INTO audio_batch_data (name, user_id, row_id, column_id, audio_batch_id) VALUES "
            .to_string();
    let mut row_counter: i64 = 0;
    let mut column_counter: i64 = 0;
    for first in payload.iter() {
        for second in first.iter() {
            write!(
                query,
                "('{}', '{}', '{}', '{}', '{}'),",
                second, user_id, row_counter, column_counter, audio_batch_id
            )
            .unwrap();
            column_counter += 1;
        }
        column_counter = 0;
        row_counter += 1;
    }

    query.pop();
    let str_query: &str = &query[..];
    sqlx::query(str_query).execute(pool).await.unwrap();
}
