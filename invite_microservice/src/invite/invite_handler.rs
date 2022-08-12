use axum::{
    extract::{rejection::JsonRejection, Extension},
    Json,
};
use axum_macros::debug_handler;
use microservice_utils::server::response::{AxumRes, AxumResult};
use openapi_rs::openapi_proc_macro::handler;
use sqlx::PgPool;
use std::sync::Arc;
use tiny_id::ShortCodeGenerator;

use crate::invite::invite::{CheckResult, EmailBody, InviteCheck, InviteLink, InviteUser, SmsBody};
use microservice_utils::jwt::extractor::AuthToken;
use microservice_utils::{server::response::into_reponse};

use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;
use openapi_rs::OpenApiFromData;

#[debug_handler]
#[handler(method = "POST",tag = "invites")]
pub async fn generate_link(
    payload: Result<Json<InviteUser>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let invite_info = payload.0;

            let mut generator = ShortCodeGenerator::new_alphanumeric(4);
            let code = generator.next_string();

            let add_user = add_invite_user(&user_id, &invite_info, &code, &pool).await;

            match add_user {
                Ok(_) => {
                    let url = format!("https://test.bhuman.ai/dl/{}", code);
                    let link = InviteLink { link: url.clone() };

                    tokio::spawn(async move {
                        send_email(&invite_info, &url).await;
                    });

                    Ok(axum::Json(AxumRes {
                        code: 200,
                        result: serde_json::json!(&link),
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

#[debug_handler]
#[handler(method = "POST",tag = "invites")]
pub async fn verify_link(
    payload: Result<Json<InviteCheck>, JsonRejection>,
    AuthToken(user_id): AuthToken,
    Extension(pool): Extension<Arc<PgPool>>,
) -> AxumResult<Json<AxumRes>> {
    match payload {
        Ok(payload) => {
            let invite_info = payload.0;

            let update_user = update_invite_user(&invite_info, &pool).await;

            match update_user {
                Ok(invitors) => {
                    let invitors = CheckResult { invitors: invitors };
                    Ok(axum::Json(AxumRes {
                        code: 200,
                        result: serde_json::json!(&invitors),
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

// Send email
pub async fn send_email(user: &InviteUser, link: &String) {
    let client = reqwest::Client::new();

    for receiver in &user.receivers {
        let message = format!("Hey {},\n\n{} {} invited you to try BHuman, the only app in the world that let's you make personalized videos at scale that look and feel completely real.\n\nWhen you sign up, both you and {} will get 250 videos for free (valued at $50 a piece). Here's a link you can use to make sure they get the credit: {}\n\nAny questions? Reply to this email and I'll give you a ring.\nDon", receiver.first_name, user.sender.first_name, user.sender.last_name, user.sender.first_name, link);
        if receiver.email.len() > 0 {
            let body = EmailBody {
                From: "don@bhuman.ai".to_string(),
                To: receiver.email.clone(),
                Subject: "Subject".to_string(),
                TextBody: message.clone(),
                MessageStream: "outbound".to_string(),
            };
            let encoded = serde_json::to_string(&body).unwrap();
            let request = client
                .post("https://api.postmarkapp.com/email")
                .header("Accept", "application/json")
                .header("Content-Type", "application/json")
                .header(
                    "X-Postmark-Server-Token",
                    "f08c3323-8c5f-4e2d-8823-7eab9a3efc8d",
                )
                .body(encoded)
                .send()
                .await
                .unwrap();

            let response = request.text().await.unwrap();

            println!("Invitation email reponse = {:?}", &response);
        }
        if receiver.phone.len() > 0 {
            let body = SmsBody {
                From: "+16319331307".to_string(),
                To: receiver.phone.clone(),
                Body: message.clone(),
            };
            let encoded1 = serde_urlencoded::to_string(&body).unwrap();
            let request = client
                            .post("https://api.twilio.com/2010-04-01/Accounts/AC12d4d2c71ec2b325a9afee5fc3aa0ade/Messages.json")
                            .header("Content-Type", "application/x-www-form-urlencoded")
                            .basic_auth("AC12d4d2c71ec2b325a9afee5fc3aa0ade", Some("01badbaf734626f048d5c5dd722e8365"))
                            .body(encoded1.clone())
                            .send()
                            .await
                            .unwrap();

            let response = request.text().await.unwrap();

            println!("Invitation sms reponse = {:?}", &response);
        }
    }
}

// Database
pub async fn add_invite_user(
    user_id: &String,
    user: &InviteUser,
    hash: &String,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    for receiver in &user.receivers {
        let _ = sqlx::query!(
            "INSERT INTO invites (user_id, invitor_name, invitee_name, email, phone, hash) VALUES ($1, $2, $3, $4, $5, $6)",
            user_id,
            user.sender.first_name,
            receiver.first_name,
            receiver.email.clone(),
            receiver.phone.clone(),
            hash,
        ).execute(pool).await?;
    }
    Ok(())
}

pub async fn update_invite_user(
    user: &InviteCheck,
    pool: &PgPool,
) -> Result<Vec<String>, sqlx::Error> {
    let mut invitors = Vec::new();
    if user.account.len() > 0 {
        let rows = sqlx::query!(
            "UPDATE invites SET status = 1 WHERE (email = $1 OR phone = $1) AND status = 0 RETURNING user_id",
            user.account,
        ).fetch_all(pool).await?;

        for row in rows {
            invitors.push(row.user_id);
        }
    }
    Ok(invitors)
}
