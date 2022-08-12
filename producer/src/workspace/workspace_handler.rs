use std::sync::Arc;
use sqlx::PgPool;
use sqlx::types::chrono::Utc;
use sqlx::types::chrono::NaiveDateTime;
use axum::extract::Extension;
use axum::{body::Body, extract::Query, extract::rejection::JsonRejection, response::Response, Json};
use axum_macros::debug_handler;

use crate::workspace::workspace::{
    CreateWorkspace,
    UpdateWorkspace,
    WorkspaceQuery,
    Workspace,
};
use crate::utils::response::{
    send_reponse,
};

// API
#[debug_handler]
pub async fn create_workspace(
    payload: Result<Json<CreateWorkspace>, JsonRejection>,
    Extension(pool): Extension<Arc<PgPool>>
) -> Result<Response<Body>, Response<Body>> {
    match payload {
        Ok(payload) => {
            let ws_info = payload.0;

            let create_ws = db_create_workspace(&ws_info, &pool).await;
            match create_ws {
                Ok(result) => {
                    let encoded = serde_json::to_string(&result).unwrap();
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
pub async fn update_workspace(
    payload: Result<Json<UpdateWorkspace>, JsonRejection>,
    Extension(pool): Extension<Arc<PgPool>>
) -> Result<Response<Body>, Response<Body>> {
    match payload {
        Ok(payload) => {
            let ws_info = payload.0;

            let create_ws = db_update_workspace(&ws_info, &pool).await;
            match create_ws {
                Ok(result) => {
                    let encoded = serde_json::to_string(&result).unwrap();
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
pub async fn get_workspace(
    params: Query<WorkspaceQuery>,
    Extension(pool): Extension<Arc<PgPool>>
) -> Result<Response<Body>, Response<Body>> {
    let user = db_get_workspace(&params.user_id, &pool).await;
    match user {
        Ok(result) => {
            println!("{:?}", result);
            let msg_str = serde_json::to_string(&result).unwrap();
            send_reponse(200, Body::from(msg_str))
        }
        Err(e) => {
            send_reponse(500, Body::from(format!("{:?}", e)))  
        }
    }
}

#[debug_handler]
pub async fn delete_workspace(
    params: Query<WorkspaceQuery>,
    Extension(pool): Extension<Arc<PgPool>>
) -> Result<Response<Body>, Response<Body>> {
    let user = db_delete_workspace(&params.user_id, &pool).await;
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
pub async fn db_create_workspace(workspace: &CreateWorkspace, pool: &PgPool) -> Result<Workspace, sqlx::Error> {
    let row = sqlx::query!(
        "INSERT INTO workspaces (
            user_id, name, description, created_at, updated_at) VALUES ($1, $2, $3, $4, $5) RETURNING *",
            workspace.user_id,
            workspace.name,
            workspace.description,
            Utc::now().naive_utc() as NaiveDateTime,
            Utc::now().naive_utc() as NaiveDateTime,
    ).fetch_one(pool).await?;
    let out_workspace = Workspace {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        description: Some(row.description).unwrap(),
        created_at: row.created_at.naive_utc(),
        updated_at: row.updated_at.naive_utc(),
    };
    Ok(out_workspace)
}

pub async fn db_update_workspace(workspace: &UpdateWorkspace, pool: &PgPool) -> Result<Workspace, sqlx::Error> {
    let row = sqlx::query!(
        "UPDATE workspaces SET user_id = $1, name = $2, description = $3, updated_at = $4 WHERE id = $5 RETURNING *",
            workspace.user_id,
            workspace.name,
            workspace.description,
            Utc::now().naive_utc() as NaiveDateTime,
            workspace.id
    ).fetch_one(pool).await?;
    let out_workspace = Workspace {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        description: Some(row.description).unwrap(),
        created_at: row.created_at.naive_utc(),
        updated_at: row.updated_at.naive_utc(),
    };
    Ok(out_workspace)
}

pub async fn db_get_workspace(user_id: &String, pool: &PgPool) -> Result<Workspace, sqlx::Error> {
    let row = sqlx::query!("SELECT * FROM workspaces WHERE user_id = $1", user_id).fetch_one(pool).await?;
    let out_workspace = Workspace {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        description: Some(row.description).unwrap(),
        created_at: row.created_at.naive_utc(),
        updated_at: row.updated_at.naive_utc(),
    };
    Ok(out_workspace)
}

pub async fn db_delete_workspace(user_id: &String, pool: &PgPool) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!("DELETE FROM workspaces WHERE user_id = $1", user_id).execute(pool).await?;
    Ok(())
}