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

use crate::models::folder::{CreateFolder, Folder, FolderOptionalId, UpdateFolder};
use crate::models::param::RequiredId;
use microservice_utils::{jwt::extractor::AuthToken, server::{grpc::check_workspace,response::{into_reponse, AxumResult, AxumRes}}};

// API
#[debug_handler]
#[handler(method = "POST",tag = "folder")]
pub async fn create_folder(
    payload: Result<Json<CreateFolder>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let folder_info = payload.0;

            let res = check_workspace(&user_id.to_string(), &folder_info.workspace_id.to_string()).await;
            match res {            
                Ok(_) => {},
                Err(e) => {
                    let ret = serde_json::json!({
                        "error": format!("{:?}", e),
                    });
                    return Err(into_reponse(404, ret))
                }
            };

            match db_create_folder(&user_id, &folder_info, 0, 0, &pool).await {
                Ok(result) => Ok(axum::Json(AxumRes{code:200, result:serde_json::json!(&result)})),
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
#[handler(method = "PUT",tag = "folder")]
pub async fn update_folder(
    payload: Result<Json<UpdateFolder>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let folder_info = payload.0;

            match db_update_folder(&user_id, &folder_info, &pool).await {
                Ok(result) => Ok(axum::Json(AxumRes{code:200, result:serde_json::json!(&result)})),
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
#[handler(method = "GET",tag = "folder")]
pub async fn get_folder(
    params: Query<FolderOptionalId>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    let folders = db_get_folder(&user_id, &params, &pool).await;
    match folders {
        Ok(result) => Ok(axum::Json(AxumRes{code:200, result:serde_json::json!(&result)})),
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
#[handler(method = "DELETE",tag = "folder")]
pub async fn delete_folder(
    payload: Result<Json<RequiredId>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let folder_info = payload.0;

            let res = db_delete_folder(&user_id, &folder_info, &pool).await;
            match res {
                Ok(_) => {
                    let ret = serde_json::json!({
                        "status": "success",
                    });
                    Ok(axum::Json(AxumRes{code:200, result:serde_json::json!(&ret)}))
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
pub async fn db_create_folder(
    user_id: &String,
    folder: &CreateFolder,
    p_videos: i64,
    g_videos: i64,
    pool: &PgPool,
) -> Result<Folder, sqlx::Error> {
    let out_folder = sqlx::query_as!(Folder, 
        r#"INSERT INTO folders (
            user_id, workspace_id, name, parent_videos, generated_videos) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
            user_id,
            folder.workspace_id,
            folder.name,
            p_videos,
            g_videos,
    ).fetch_one(pool).await?;    
    Ok(out_folder)
}

pub async fn db_update_folder(
    user_id: &String,
    folder: &UpdateFolder,
    pool: &PgPool,
) -> Result<Folder, sqlx::Error> {
    let out_folder = sqlx::query_as!(Folder, 
        r#"UPDATE folders SET name = $1, updated_at = $2 WHERE id = $3 AND user_id = $4 RETURNING *"#,
        folder.name,
        Utc::now().naive_utc() as NaiveDateTime,
        folder.id,
        user_id
    )
    .fetch_one(pool)
    .await?;    
    Ok(out_folder)
}

pub async fn db_get_folder(
    user_id: &String,
    params: &FolderOptionalId,
    pool: &PgPool,
) -> Result<Vec<Folder>, sqlx::Error> {
    let mut query = "SELECT * FROM folders WHERE ".to_string();
    write!(query, "user_id = '{}' ", user_id).unwrap();

    if let Some(params_id) = params.id {
        write!(query, "And id = '{}' ", params_id).unwrap();
    }
    if let Some(params_workspace_id) = params.workspace_id {
        write!(query, "And workspace_id = '{}' ", params_workspace_id).unwrap();
    }

    let str_query: &str = &query[..];
    let folders = sqlx::query_as::<_, Folder>(str_query)
        .fetch_all(pool)
        .await?;
    Ok(folders)
}

pub async fn db_delete_folder(
    user_id: &String,
    params: &RequiredId,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    match db_check_v_instance(&user_id, &params.id, &pool).await {
        Ok(_) => Err(sqlx::Error::Protocol(
            "Can't delete, this folder have video instances".to_string(),
        )),
        Err(_) => {
            let _ = sqlx::query!(
                "DELETE FROM folders WHERE id = $1 AND user_id = $2",
                params.id,
                user_id
            )
            .execute(pool)
            .await?;
            Ok(())
        }
    }
}

pub async fn db_check_v_instance(
    user_id: &String,
    folder_id: &Uuid,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        "SELECT id FROM video_instances WHERE folder_id = $1 AND user_id = $2",
        folder_id,
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(())
}
