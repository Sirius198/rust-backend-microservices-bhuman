use schemars::JsonSchema;
use serde::{Serialize,Deserialize};
use serde_json::Value;

pub struct ResponseError(pub anyhow::Error);

impl From<anyhow::Error> for ResponseError {
    fn from(error: anyhow::Error) -> Self {
        Self(error)
    }
}

impl axum::response::IntoResponse for ResponseError {
    fn into_response(self) -> axum::response::Response {
        let status_code = if let Some(error) = self.0.downcast_ref::<ApiError>() {
            match error {
                ApiError::NotFound => axum::http::StatusCode::NOT_FOUND,
                ApiError::BadRequest => axum::http::StatusCode::BAD_REQUEST,
                ApiError::InternalServerError => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            }
        } else {
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        };
        (status_code, self.0.to_string()).into_response()
    }
}

pub type AxumResult<T> = Result<T, ResponseError>;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("bad request")]
    BadRequest,
    #[error("Internal Server error")]
    InternalServerError,
}

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub struct AxumRes {
    pub result: Value,
    pub code: i64
}

pub fn into_reponse(code: i64, body: serde_json::Value) -> ResponseError {
    let value = serde_json::json!({
        "code": code,
        "result": body,
    });

    let code = match code {
        404 => ApiError::NotFound,
        400 => ApiError::BadRequest,
        500 => ApiError::InternalServerError,
        _ => ApiError::InternalServerError
    };

    ResponseError(anyhow::anyhow!(code).context(value.to_string()))
}