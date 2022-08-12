
use sqlx::PgPool;
use std::sync::Arc;
use tiny_id::ShortCodeGenerator;
use axum::{extract::Extension, body::Body, extract::rejection::JsonRejection, response::Response, Json};
use axum_macros::debug_handler;

use crate::invite::invite::{
    InviteUser,
    InviteLink,
    InviteCheck,
    CheckResult,
    EmailBody,
    SmsBody,
};

use crate::utils::{
    response::send_reponse,
};

#[debug_handler]
pub async fn generate_link(
     payload: Result<Json<InviteUser>, JsonRejection>,
     Extension(pool): Extension<Arc<PgPool>>
) -> Result<Response<Body>, Response<Body>> {
    match payload {
        Ok(payload) => {
            let invite_info = payload.0;

            let mut generator = ShortCodeGenerator::new_alphanumeric(4);
            let code = generator.next_string();

            let add_user = add_invite_user(
                &invite_info, 
                &code,
                &pool
            ).await;

            match add_user {
                Ok(_) => {
                    let url = format!("http://localhost:5000/dl/{}", code);
                    let link = InviteLink {
                        link: url.clone(),
                    };

                    tokio::spawn(async move {                                                                
                        send_email(&invite_info, &url).await;
                    });

                    let encoded = serde_json::to_string(&link).unwrap();
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
pub async fn verify_link(
     payload: Result<Json<InviteCheck>, JsonRejection>,
     Extension(pool): Extension<Arc<PgPool>>
) -> Result<Response<Body>, Response<Body>> {
    match payload {
        Ok(payload) => {
            let invite_info = payload.0;

            let update_user = update_invite_user(
                &invite_info, 
                &pool
            ).await;

            match update_user {
                Ok(invitors) => {
                    let invitors = CheckResult {
                        invitors: invitors,
                    };
                    let encoded = serde_json::to_string(&invitors).unwrap();
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
                            .header("X-Postmark-Server-Token", "f08c3323-8c5f-4e2d-8823-7eab9a3efc8d")
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
    user: &InviteUser,
    hash: &String,    
    pool: &PgPool
) -> Result<(), sqlx::Error> {
    for receiver in &user.receivers {
        let _ = sqlx::query!(
            "INSERT INTO invites (user_id, email, phone, hash) VALUES ($1, $2, $3, $4)",
            user.sender.user_id,
            receiver.email.clone(),
            receiver.phone.clone(),
            hash,
        ).execute(pool).await?;
    }    
    Ok(())
}

pub async fn update_invite_user(
    user: &InviteCheck,
    pool: &PgPool
) -> Result<Vec<String>, sqlx::Error> {
    let mut invitors = Vec::new();
    if user.account.len() > 0 {
        let rows = sqlx::query!(
            "UPDATE invites SET used = 1 WHERE email = $1 OR phone = $2 RETURNING user_id",
            user.account,
            user.account,
        ).fetch_all(pool).await?;
        
        for row in rows {
            invitors.push(row.user_id);
        }
    }
    Ok(invitors)
}