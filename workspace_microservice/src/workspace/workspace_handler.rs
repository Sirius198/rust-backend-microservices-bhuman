use std::sync::Arc;
use microservice_utils::server::response::{AxumResult, AxumRes};
use openapi_rs::openapi_proc_macro::handler;
use sqlx::PgPool;
use sqlx::types::chrono::Utc;
use sqlx::types::chrono::NaiveDateTime;
use axum::extract::Extension;
use axum::{extract::{Query, rejection::JsonRejection}, Json};
use axum_macros::debug_handler;
use rdkafka::producer::FutureProducer;
use std::fmt::Write;
use uuid::Uuid;
use tonic::async_trait;

use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;
use openapi_rs::OpenApiFromData;

use crate::workspace::workspace_type::{
    CreateWorkspace,
    UpdateWorkspace,
    AddToWorkspace,
    RemoveFromWorkspace,
    Workspace,
};
use crate::workspace::param::{RequiredId, OptionalId};
use microservice_utils::server::grpc::{
    add_workspace_id,
    remove_workspace_id,
};
use microservice_utils::{jwt::{extractor::AuthToken}, server::response::into_reponse};
use crate::producer::{
    producer::produce,
    ws_message::WsMessage,
};

use crate::workspace_service::workspace_service_server::WorkspaceService;
use crate::workspace_service::{WorkspaceInfo, WorkspaceStatus};

// gRPC
pub struct MyWorkspaceService {
    pool: PgPool
}

impl MyWorkspaceService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

#[async_trait]
impl WorkspaceService for MyWorkspaceService {
    async fn check_workspace(
        &self,
        request: tonic::Request<WorkspaceInfo>,
    ) -> Result<tonic::Response<WorkspaceStatus>, tonic::Status> {
        
        let req: WorkspaceInfo = request.into_inner();
        println!("Check Workspace {:?}", req);

        match db_check_workspace(&req, &self.pool).await {
            Ok(ret) => {
                if ret.is_empty() == false {
                    Ok(tonic::Response::new(WorkspaceStatus {
                        status: "success".to_string(),
                    }))
                } else {
                    Ok(tonic::Response::new(WorkspaceStatus {
                        status: "not exist".to_string(),
                    }))
                }                
            }
            Err(e) => {
                Ok(tonic::Response::new(WorkspaceStatus {
                    status: e.to_string(),
                }))
            }
        }
    }
}

// API
#[debug_handler]
#[handler(method = "POST",tag = "workspace")]
pub async fn create_workspace(
    payload: Result<Json<CreateWorkspace>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    
    match payload {
        Ok(payload) => {
            let ws_info = payload.0;

            let create_ws = db_create_workspace(&user_id, &ws_info, &pool).await;
            match create_ws {
                Ok(result) => {
                    // to grpc
                    let _ = add_workspace_id(&result.user_id, &result.workspace_id).await;

                    Ok(axum::Json(AxumRes{code:200, result:serde_json::json!(&result)}))
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
#[handler(method = "PUT",tag = "workspace")]
pub async fn update_workspace(
    payload: Result<Json<UpdateWorkspace>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let ws_info = payload.0;

            let create_ws = db_update_workspace(&user_id, &ws_info, &pool).await;
            match create_ws {
                Ok(result) => {
                    Ok(axum::Json(AxumRes{code:200, result:serde_json::json!(&result)}))
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
#[handler(method = "GET",tag = "workspace")]
pub async fn get_workspace(
    params: Query<OptionalId>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    let workspace = db_get_workspace(&user_id, &params, &pool).await;
    match workspace {
        Ok(result) => {
            Ok(axum::Json(AxumRes{code:200, result:serde_json::json!(&result)}))
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
#[handler(method = "DELETE",tag = "workspace")]
pub async fn delete_workspace(
    payload: Result<Json<RequiredId>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let ws_info = payload.0;

            let users = db_delete_workspace(&user_id, &ws_info, &pool).await;
            match users {
                Ok(result) => {
                    // to grpc
                    for peer_id in result {
                        let _ = remove_workspace_id(&peer_id, &ws_info.id).await;
                    }

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

#[debug_handler]
#[handler(method = "POST",tag = "workspace_utils")]
pub async fn add_to_workspace(
    payload: Result<Json<AddToWorkspace>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
    Extension(producer): Extension<Arc<FutureProducer>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let ws_info = payload.0;

            let add_ws = db_add_to_workspace(&user_id, &ws_info, &pool).await;
            match add_ws {
                Ok(result) => {
                    // to grpc
                    let _ = add_workspace_id(&result.user_id, &result.workspace_id).await;

                    // to broker
                    let message = WsMessage {
                        user_id: ws_info.peer_id.clone(),
                        message_type: "add_workspace".to_string(),
                        message: serde_json::to_string(&ws_info).unwrap(),
                    };
                    let msg_str = serde_json::to_string(&message).unwrap();  
                
                    let produce = produce(&msg_str, &producer).await;
                    println!("Message sent to kafka {:?}", produce);

                    // to frontend
                    Ok(axum::Json(AxumRes{code:200, result:serde_json::json!(&result)}))
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
#[handler(method = "DELETE",tag = "workspace_utils")]
pub async fn remove_from_workspace(
    payload: Result<Json<RemoveFromWorkspace>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
    Extension(producer): Extension<Arc<FutureProducer>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let ws_info = payload.0;

            let remove_ws = db_remove_from_workspace(&user_id, &ws_info, &pool).await;
            match remove_ws {
                Ok(_) => {
                    // to grpc
                    let _ = remove_workspace_id(&ws_info.peer_id, &ws_info.id).await;

                    // to broker
                    let message = WsMessage {
                        user_id: ws_info.peer_id.clone(),
                        message_type: "remove_workspace".to_string(),
                        message: serde_json::to_string(&ws_info).unwrap(),
                    };
                    let msg_str = serde_json::to_string(&message).unwrap();  

                    let produce = produce(&msg_str, &producer).await;
                    println!("Message sent to kafka {:?}", produce);

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
pub async fn db_create_workspace(user_id: &String, workspace: &CreateWorkspace, pool: &PgPool) -> Result<Workspace, sqlx::Error> {
    let out_workspace = sqlx::query_as!(Workspace, 
        r#"INSERT INTO workspaces (
            user_id, name, role, description) VALUES ($1, $2, $3, $4) RETURNING *"#,
            user_id,
            workspace.name,
            workspace.role,
            workspace.description
    ).fetch_one(pool).await?;
    Ok(out_workspace)
}

pub async fn db_update_workspace(user_id: &String, workspace: &UpdateWorkspace, pool: &PgPool) -> Result<Workspace, sqlx::Error> {
    let out_workspace = sqlx::query_as!(Workspace, 
        r#"UPDATE workspaces SET name = $1, role = $2, description = $3, updated_at = $4 WHERE user_id = $5 AND workspace_id = $6 RETURNING *"#,
            workspace.name,
            workspace.role,
            workspace.description,
            Utc::now().naive_utc() as NaiveDateTime,
            user_id,
            workspace.id
    ).fetch_one(pool).await?;
    Ok(out_workspace)
}

pub async fn db_get_workspace(user_id: &String, params: &OptionalId, pool: &PgPool) -> Result<Vec<Workspace>, sqlx::Error> {
    let mut query = "SELECT * FROM workspaces WHERE ".to_string();
    write!(query, "user_id = '{}' ", user_id).unwrap();

    if let Some(params_id) = &params.id {
        write!(query, "And workspace_id = '{}' ", params_id).unwrap();
    }

    let str_query: &str = &query[..];
    let workspaces = sqlx::query_as::<_, Workspace>(str_query)
        .fetch_all(pool)
        .await?;
    Ok(workspaces)    
}

pub async fn db_get_workspace_by_id(user_id: &String, id: &Uuid, pool: &PgPool) -> Result<Workspace, sqlx::Error> {
    let workspace = sqlx::query_as!(Workspace, r#"SELECT * FROM workspaces WHERE user_id = $1 AND workspace_id = $2"#, user_id, id).fetch_one(pool).await?;
    Ok(workspace)    
}

pub async fn db_delete_workspace(_user_id: &String, params: &RequiredId, pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let mut users = Vec::new();
    let rows = sqlx::query!("SELECT user_id FROM workspaces WHERE workspace_id = $1", params.id).fetch_all(pool).await?;
    for row in rows {
        users.push(row.user_id);
    }
    
    let _ = sqlx::query!("DELETE FROM workspaces WHERE workspace_id = $1", params.id).execute(pool).await?;    
    Ok(users)
}

pub async fn db_add_to_workspace(user_id: &String, params: &AddToWorkspace, pool: &PgPool) -> Result<Workspace, sqlx::Error> {
    let ws = db_get_workspace_by_id(&user_id, &params.id, &pool).await;
    match ws {
        Ok(result) => {
            let out_workspace = sqlx::query_as!(Workspace, 
                r#"INSERT INTO workspaces (
                    workspace_id, user_id, name, role, description) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
                    params.id,
                    params.peer_id,
                    result.name,
                    params.role,
                    result.description
            ).fetch_one(pool).await?;
            Ok(out_workspace)
        }
        Err(e) => {
            println!("{:?}", e.to_string());
            Err(e)                
        }
    }    
}

pub async fn db_remove_from_workspace(_user_id: &String, params: &RemoveFromWorkspace, pool: &PgPool) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!("DELETE FROM workspaces WHERE workspace_id = $1 AND user_id = $2", 
        params.id,
        params.peer_id,
    ).execute(pool).await?;
    Ok(())
}

pub async fn db_check_workspace(
    req: &WorkspaceInfo,
    pool: &PgPool,
) -> Result<String, sqlx::Error> {
    let row = sqlx::query!("SELECT workspace_id FROM workspaces where user_id = $1 AND workspace_id = $2", req.user_id, Uuid::parse_str(&req.workspace_id).unwrap()).fetch_one(pool).await?;
    Ok(row.workspace_id.to_string())
}