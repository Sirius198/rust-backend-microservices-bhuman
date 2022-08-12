use anyhow::Context;
use axum::{
    extract::{rejection::JsonRejection, Extension},
    Json,
};
use axum_macros::debug_handler;
use openapi_rs::openapi_proc_macro::handler;
use sqlx::PgPool;
use std::sync::Arc;
use tonic::async_trait;
use tonic::{Code, Status};

use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;
use openapi_rs::OpenApiFromData;

use microservice_utils::{
    jwt::auth::{create_token, Token},
    server::response::{into_reponse, AxumRes,AxumResult},
};

use crate::models::auth::{Email, PhoneNumber, Shopify, StytchOTP, StytchToken, StytchAuth};

use crate::auth_service::auth_service_server::AuthService;
use crate::auth_service::{CheckTokenRequest, CheckTokenResponse, TokenRefreshRequest, TokenRefreshResponse, CheckShopifyToken, ShopifyTokenResponse};

lazy_static! {
    static ref BASE_URL: String = "https://api.stytch.com/v1".to_owned();
    static ref PROJECT_ID: String = "project-live-e3b801f1-b65a-44b1-ac74-006cbe4cebe1".to_owned();
    static ref SECRET_ID: String = "secret-live-Abpes6Vi7UPs3vpxH6vA3LZtivK_zzEXyAs=".to_owned();
}

// gRPC
pub struct MyAuthService {
    pool: PgPool
}

impl MyAuthService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

#[async_trait]
impl AuthService for MyAuthService {
    async fn check_token(
        &self,
        request: tonic::Request<CheckTokenRequest>,
    ) -> Result<tonic::Response<CheckTokenResponse>, tonic::Status> {

        let req: CheckTokenRequest = request.into_inner();
        println!("Check Token {:?}", req);

        let _ = db_check_token(&req, &self.pool)
            .await
            .with_context(|| anyhow::anyhow!("Access token does not exist"))
            .map_err(|e| Status::new(Code::Internal, format!("{:?}", e)))?;

        Ok(tonic::Response::new(CheckTokenResponse {
            status: "success".to_string(),
        }))
    }

    async fn refresh_token(
        &self,
        request: tonic::Request<TokenRefreshRequest>,
    ) -> Result<tonic::Response<TokenRefreshResponse>, tonic::Status> {

        let req: TokenRefreshRequest = request.into_inner();
        println!("Refresh Token {:?}", req);

        let _ = db_check_refresh_token(&req, &self.pool)
            .await
            .with_context(|| anyhow::anyhow!("Refresh token does not exist"))
            .map_err(|e| Status::new(Code::Internal, format!("{:?}", e)))?;

        let token = create_token(&req.user_id);
        match db_update_token(&req.user_id, &token.access_token, &self.pool).await {
            Ok(_) => {
                Ok(tonic::Response::new(TokenRefreshResponse {
                    status: "success".to_string(),
                    access_token: token.access_token,
                }))
            }
            Err(e) => {
                Err(Status::new(Code::Internal, format!("{:?}", e)))
            }
        }
    }

    async fn get_shopify_token(
        &self,
        request: tonic::Request<CheckShopifyToken>,
    ) -> Result<tonic::Response<ShopifyTokenResponse>, tonic::Status> {

        let req: CheckShopifyToken = request.into_inner();
        println!("Request Shopify Token {:?}", req);

        let ret = db_get_shopify_token(&req, &self.pool)
            .await
            .with_context(|| anyhow::anyhow!("Shopify token does not exist"))
            .map_err(|e| Status::new(Code::Internal, format!("{:?}", e)))?;

        Ok(tonic::Response::new(ShopifyTokenResponse {
            status: "success".to_string(),
            token: ret.to_string(),
        }))
    }
}

// API
#[debug_handler]
#[handler(method = "POST",tag = "auth")]
pub async fn email_auth_link(
    payload: Result<Json<Email>, JsonRejection>
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let mut email = payload.0;
            email.expiration_minutes = Some(5);

            let client = reqwest::Client::new();
            let request = client
                .post(format!(
                    "{}/magic_links/email/login_or_create",
                    BASE_URL.to_string()
                ))
                .header("Content-Type", "application/json")
                .basic_auth(PROJECT_ID.to_string(), Some(SECRET_ID.to_string()))
                .body(serde_json::to_string(&email).unwrap())
                .send()
                .await
                .unwrap();

            let response = request.text().await.unwrap();

            let v: serde_json::Value = serde_json::from_str(&response).unwrap();
            let code = v["status_code"].as_i64().unwrap();
            if code == 200 {
                let ret = serde_json::json!({
                    "user_created": v["user_created"],
                    "method_id": v["email_id"],
                    "user_id": v["user_id"],
                });
                Ok(axum::Json(AxumRes{code: 200, result: ret}))
            } else {
                let ret = serde_json::json!({
                    "error": v["error_message"],
                });
                Err(into_reponse(code, ret))
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
#[handler(method = "POST",tag = "auth")]
pub async fn email_auth_otp(
    payload: Result<Json<Email>, JsonRejection>
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let mut email = payload.0;
            email.expiration_minutes = Some(5);

            let client = reqwest::Client::new();
            let request = client
                .post(format!("{}/otps/email/login_or_create", BASE_URL.to_string()))
                .header("Content-Type", "application/json")
                .basic_auth(PROJECT_ID.to_string(), Some(SECRET_ID.to_string()))
                .body(serde_json::to_string(&email).unwrap())
                .send()
                .await
                .unwrap();

            let response = request.text().await.unwrap();

            let v: serde_json::Value = serde_json::from_str(&response).unwrap();
            let code = v["status_code"].as_i64().unwrap();
            if code == 200 {
                let ret = serde_json::json!({
                    "user_created": v["user_created"],
                    "method_id": v["email_id"],
                    "user_id": v["user_id"],
                });
                Ok(axum::Json(AxumRes{code: 200, result: ret}))
            } else {
                let ret = serde_json::json!({
                    "error": v["error_message"],
                });
                Err(into_reponse(code, ret))
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
#[handler(method = "POST",tag = "auth")]
pub async fn phone_auth_otp(
    payload: Result<Json<PhoneNumber>, JsonRejection>
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let mut phone = payload.0;
            phone.expiration_minutes = Some(5);
            phone.e164_format();

            let client = reqwest::Client::new();
            let request = client
                .post(format!("{}/otps/sms/login_or_create", BASE_URL.to_string()))
                .header("Content-Type", "application/json")
                .basic_auth(PROJECT_ID.to_string(), Some(SECRET_ID.to_string()))
                .body(serde_json::to_string(&phone).unwrap())
                .send()
                .await
                .unwrap();

            let response = request.text().await.unwrap();

            let v: serde_json::Value = serde_json::from_str(&response).unwrap();
            let code = v["status_code"].as_i64().unwrap();
            if code == 200 {
                let ret = serde_json::json!({
                    "user_created": v["user_created"],
                    "method_id": v["phone_id"],
                    "user_id": v["user_id"],
                });
                Ok(axum::Json(AxumRes{code: 200, result: ret}))
            } else {
                let ret = serde_json::json!({
                    "error": v["error_message"],
                });
                Err(into_reponse(code, ret))
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
#[handler(method = "POST",tag = "auth")]
pub async fn shopify_auth_otp(
    payload: Result<Json<Shopify>, JsonRejection>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let shopify_info = payload.0;
            
            let email = Email {
                email: shopify_info.email.clone(),
                expiration_minutes: Some(5),
            };

            let client = reqwest::Client::new();
            let request = client
                .post(format!("{}/otps/email/login_or_create", BASE_URL.to_string()))
                .header("Content-Type", "application/json")
                .basic_auth(PROJECT_ID.to_string(), Some(SECRET_ID.to_string()))
                .body(serde_json::to_string(&email).unwrap())
                .send()
                .await
                .unwrap();

            let response = request.text().await.unwrap();

            let v: serde_json::Value = serde_json::from_str(&response).unwrap();
            let code = v["status_code"].as_i64().unwrap();
            if code == 200 {

                match db_create_shopify_auth(&v["user_id"].as_str().unwrap().to_string(), &shopify_info, &pool).await {
                    Ok(_) => {
                        let ret = serde_json::json!({
                            "user_created": v["user_created"],
                            "method_id": v["email_id"],
                            "user_id": v["user_id"],
                        });
                        Ok(axum::Json(AxumRes{code: 200, result: ret}))
                    }
                    Err(e) => {
                        println!("{:?}", e.to_string());
                        let ret = serde_json::json!({
                            "error": format!("{:?}", e),
                        });
                        Err(into_reponse(500, ret))
                    }
                }
            } else {
                let ret = serde_json::json!({
                    "error": v["error_message"],
                });
                Err(into_reponse(code, ret))
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
#[handler(method = "POST",tag = "auth_verify")]
pub async fn email_verify_link(
    payload: Result<Json<StytchToken>, JsonRejection>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let token = payload.0;
            let client = reqwest::Client::new();
            let request = client
                .post(format!("{}/magic_links/authenticate", BASE_URL.to_string()))
                .header("Content-Type", "application/json")
                .basic_auth(PROJECT_ID.to_string(), Some(SECRET_ID.to_string()))
                .body(serde_json::to_string(&token).unwrap())
                .send()
                .await
                .unwrap();

            let response = request.text().await.unwrap();

            let v: serde_json::Value = serde_json::from_str(&response).unwrap();
            let code = v["status_code"].as_i64().unwrap();
            if code == 200 {
                let token = create_token(&v["user_id"].as_str().unwrap().to_string());
                match db_create_auth(
                    &v["user_id"].as_str().unwrap().to_string(),
                    &"Email".to_string(),
                    &token,
                    &pool,
                )
                .await
                {
                    Ok(_) => {
                        let ret = serde_json::json!({
                            "user_id": v["user_id"],
                            "token": token,
                        });
                        Ok(axum::Json(AxumRes{code: 200, result: ret}))
                    }
                    Err(e) => {
                        println!("{:?}", e.to_string());
                        let ret = serde_json::json!({
                            "error": format!("{:?}", e),
                        });
                        Err(into_reponse(500, ret))
                    }
                }
            } else {
                println!("{:?}", v["error_message"]);
                let ret = serde_json::json!({
                    "error": v["error_message"],
                });
                Err(into_reponse(code, ret))
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
#[handler(method = "POST",tag = "auth_verify")]
pub async fn email_verify_otp(
    payload: Result<Json<StytchOTP>, JsonRejection>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let token = payload.0;
            let client = reqwest::Client::new();
            let request = client
                .post(format!("{}/otps/authenticate", BASE_URL.to_string()))
                .header("Content-Type", "application/json")
                .basic_auth(PROJECT_ID.to_string(), Some(SECRET_ID.to_string()))
                .body(serde_json::to_string(&token).unwrap())
                .send()
                .await
                .unwrap();

            let response = request.text().await.unwrap();

            let v: serde_json::Value = serde_json::from_str(&response).unwrap();
            let code = v["status_code"].as_i64().unwrap();
            if code == 200 {
                let token = create_token(&v["user_id"].as_str().unwrap().to_string());
                match db_create_auth(
                    &v["user_id"].as_str().unwrap().to_string(),
                    &"Email".to_string(),
                    &token,
                    &pool,
                )
                .await
                {
                    Ok(_) => {
                        let ret = serde_json::json!({
                            "user_id": v["user_id"],
                            "token": token,
                        });
                        Ok(axum::Json(AxumRes{code: 200, result: ret}))
                    }
                    Err(e) => {
                        println!("{:?}", e.to_string());
                        let ret = serde_json::json!({
                            "error": format!("{:?}", e),
                        });
                        Err(into_reponse(500, ret))
                    }
                }
            } else {
                let ret = serde_json::json!({
                    "error": v["error_message"],
                });
                Err(into_reponse(code, ret))
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
#[handler(method = "POST",tag = "auth_verify")]
pub async fn phone_verify_otp(
    payload: Result<Json<StytchOTP>, JsonRejection>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let token = payload.0;
            let client = reqwest::Client::new();
            let request = client
                .post(format!("{}/otps/authenticate", BASE_URL.to_string()))
                .header("Content-Type", "application/json")
                .basic_auth(PROJECT_ID.to_string(), Some(SECRET_ID.to_string()))
                .body(serde_json::to_string(&token).unwrap())
                .send()
                .await
                .unwrap();

            let response = request.text().await.unwrap();

            let v: serde_json::Value = serde_json::from_str(&response).unwrap();
            let code = v["status_code"].as_i64().unwrap();
            if code == 200 {
                let token = create_token(&v["user_id"].as_str().unwrap().to_string());
                match db_create_auth(
                    &v["user_id"].as_str().unwrap().to_string(),
                    &"Phone".to_string(),
                    &token,
                    &pool,
                )
                .await
                {
                    Ok(_) => {
                        let ret = serde_json::json!({
                            "user_id": v["user_id"],
                            "token": token,                            
                        });
                        Ok(axum::Json(AxumRes{code: 200, result: ret}))
                    }
                    Err(e) => {
                        println!("{:?}", e.to_string());
                        let ret = serde_json::json!({
                            "error": format!("{:?}", e),
                        });
                        Err(into_reponse(500, ret))
                    }
                }
            } else {
                let ret = serde_json::json!({
                    "error": v["error_message"],
                });
                Err(into_reponse(code, ret))
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

// Google
// https://api.stytch.com/v1/public/oauth/google/start?public_token=public-token-live-3780acd3-6da2-4987-84e0-abc2207f6508&custom_scopes=https://www.googleapis.com/auth/contacts.readonly

// Outlook
// https://api.stytch.com/v1/public/oauth/microsoft/start?public_token=public-token-live-3780acd3-6da2-4987-84e0-abc2207f6508&custom_scopes=Contacts.Read

// LinkedIn
// https://api.stytch.com/v1/public/oauth/linkedin/start?public_token=public-token-live-3780acd3-6da2-4987-84e0-abc2207f6508

#[debug_handler]
#[handler(method = "POST",tag = "auth_verify")]
pub async fn oauth_verify(
    payload: Result<Json<StytchAuth>, JsonRejection>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let token_info = payload.0;
            let token = StytchToken {
                token: token_info.token,
            };

            let client = reqwest::Client::new();
            let request = client
                .post(format!("{}/oauth/authenticate", BASE_URL.to_string()))
                .header("Content-Type", "application/json")
                .basic_auth(PROJECT_ID.to_string(), Some(SECRET_ID.to_string()))
                .body(serde_json::to_string(&token).unwrap())
                .send()
                .await
                .unwrap();

            let response = request.text().await.unwrap();

            let v: serde_json::Value = serde_json::from_str(&response).unwrap();
            let code = v["status_code"].as_i64().unwrap();
            if code == 200 {
                if let Some(user_id) = &token_info.user_id {
                    let ret = serde_json::json!({
                        "user_id": user_id,
                        "id_token": v["provider_values"]["access_token"]
                    });
                    Ok(axum::Json(AxumRes{code: 200, result: ret}))
                } else {
                    let token = create_token(&v["user_id"].as_str().unwrap().to_string());
                    match db_create_auth(
                        &v["user_id"].as_str().unwrap().to_string(),
                        &v["provider_type"].as_str().unwrap().to_string(),
                        &token,
                        &pool,
                    )
                    .await
                    {
                        Ok(_) => {
                            let ret = serde_json::json!({
                                "user_id": v["user_id"],
                                "token": token,
                                "id_token": v["provider_values"]["access_token"],
                                "user": v["user"]
                            });
                            Err(into_reponse(200, ret))
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
            } else {
                let ret = serde_json::json!({
                    "error": v["error_message"],
                });
                Err(into_reponse(code, ret))
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
pub async fn db_create_auth(
    user_id: &String,
    provider: &String,
    token: &Token,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        "INSERT INTO auth (user_id, access_token, refresh_token, provider_type) VALUES ($1, $2, $3, $4) ON CONFLICT (user_id) DO UPDATE SET access_token = $2, refresh_token = $3, provider_type = $4",
        user_id,
        token.access_token,
        token.refresh_token,
	    provider,        
    ).execute(pool).await?;
    Ok(())
}

pub async fn db_create_shopify_auth(
    user_id: &String,
    shopify: &Shopify,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        "INSERT INTO shopify_auth (user_id, token, email) VALUES ($1, $2, $3) ON CONFLICT (user_id) DO UPDATE SET token = $2, email = $3",
        user_id,
        shopify.token,
        shopify.email,        
    ).execute(pool).await?;
    Ok(())
}

pub async fn db_update_token(
    user_id: &String, 
    access_token: &String, 
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!("UPDATE auth SET access_token = $1 WHERE user_id = $2", access_token, user_id).execute(pool).await?;
    Ok(())
}

pub async fn db_check_token(
    req: &CheckTokenRequest,
    pool: &PgPool,
) -> Result<String, sqlx::Error> {
    let row = sqlx::query!("SELECT user_id FROM auth where user_id = $1 AND access_token = $2", req.user_id, req.access_token).fetch_one(pool).await?;
    Ok(row.user_id)
}

pub async fn db_check_refresh_token(
    req: &TokenRefreshRequest,
    pool: &PgPool,
) -> Result<String, sqlx::Error> {
    let row = sqlx::query!("SELECT user_id FROM auth where user_id = $1 AND refresh_token = $2", req.user_id, req.refresh_token).fetch_one(pool).await?;
    Ok(row.user_id)
}

pub async fn db_get_shopify_token(
    req: &CheckShopifyToken,
    pool: &PgPool,
) -> Result<String, sqlx::Error> {
    let row = sqlx::query!("SELECT token FROM shopify_auth where user_id = $1", req.user_id).fetch_one(pool).await?;
    Ok(row.token)
}