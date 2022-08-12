use std::sync::Arc;

use axum::Extension;
use axum::Json;
use axum_macros::debug_handler;
use microservice_utils::jwt::extractor::AuthToken;
use openapi_rs::openapi_proc_macro::{handler};

use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;

use microservice_utils::server::response::{into_reponse, AxumRes, AxumResult};

// use openssl::pkey::PKey;
use openssl::hash::{hash, MessageDigest};
use openssl::rand::rand_bytes;
// use openssl::rsa::Rsa;
use serde_json::json;
// use std::str;

use nanoid::nanoid;
use sqlx::PgPool;

const CHARS: &[char; 63] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', 'A',
    'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
    'U', 'V', 'W', 'X', 'Y', 'Z',
];

fn random(size: usize) -> Vec<u8> {
    let mut buf = vec![0; size];
    rand_bytes(&mut buf).expect("failed to generate random bytes");

    buf.to_vec()
}

#[debug_handler]
#[handler(method = "POST", tag = "keygen")]
pub async fn generate_keypairs(
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    let client_id = nanoid!(37, CHARS, random);

    let bytes = hash(MessageDigest::sha256(), client_id.as_bytes())
        .map_err(|e| into_reponse(500, json!(e.to_string())))?;

    let client_secret = hex::encode(bytes);

    let _ = sqlx::query!(
        "INSERT INTO generated_keys(user_id,client_id,client_secret) VALUES($1,$2,$3)",
        user_id,
        client_id,
        client_secret
    )
    .execute(&*pool)
    .await
    .map_err(|e| into_reponse(500, json!(e.to_string())))?;

    Ok(Json(AxumRes {
        code: 200,
        result: json!({ "client_id": client_id,"client_secret": client_secret }),
    }))
}
