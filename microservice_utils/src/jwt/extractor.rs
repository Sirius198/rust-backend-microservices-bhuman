use axum::{async_trait, extract::RequestParts, TypedHeader};
use headers::{authorization::Bearer, Authorization};
use okapi::openapi3::{
    SecurityRequirement, SecurityScheme, SecuritySchemeData,
};
use openapi_rs::{
    gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::auth::jwt_auth;

#[derive(Serialize, Default, Deserialize, JsonSchema)]
pub struct AuthToken(pub String);

#[async_trait]
impl<T> axum::extract::FromRequest<T> for AuthToken
where
    T: Send,
{
    type Rejection = String;

    async fn from_request(req: &mut RequestParts<T>) -> Result<Self, Self::Rejection> {
        let cookies = TypedHeader::<Authorization<Bearer>>::from_request(req)
            .await
            .map_err(|e| {
                let ret = serde_json::json!({
                    "code": 404,
                    "body": format!("{:?}", e),
                });
                ret.to_string()
            })?;
        jwt_auth(cookies)
            .await
            .map_err(|e| {
                let ret = serde_json::json!({
                    "code": 404,
                    "body": format!("{:?}", e),
                });
                ret.to_string()
            })
            .map(|id| AuthToken(id))
    }
}

impl<T> OpenApiFromRequest<T> for AuthToken
where
    T: Send,
{
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> anyhow::Result<RequestHeaderInput> {
        // Setup global requirement for Security scheme
        let security_scheme = SecurityScheme {
            description: Some("Requires an Access Token".to_owned()),
            // Setup data requirements.
            data: SecuritySchemeData::Http {
                // Other flows are very similar.
                // For more info see: https://swagger.io/docs/specification/authentication/oauth2/
                scheme: "bearer".into(),
                bearer_format: Some("JWT".into())
                // bearer_format:Some("JWT".into()),
            },
            // Add example data for RapiDoc
            extensions: okapi::map! {
            },
        };
        // Add the requirement for this route/endpoint
        // This can change between routes.
        let mut security_req = SecurityRequirement::new();

        security_req.insert("Bearer".to_owned(), Vec::new());

        // Each security requirement needs to be met before access is allowed.
        // These vvvvvvv-----^^^^^^^^^^ values need to match exactly!
        Ok(RequestHeaderInput::Security(
            "Bearer".to_owned(),
            security_scheme,
            security_req,
        ))
    }
}
