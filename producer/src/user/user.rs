use uuid::Uuid;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::chrono::NaiveDateTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,                    // user id (stytch user id)
    pub first_name: String,                 // first name
    pub last_name: String,                  // last name
    pub username: String,                   // username
    pub email: String,                      // email
    pub dob: NaiveDateTime,                 // birthday
    pub last_at: NaiveDateTime,             // last login date
    pub two_fator: Option<bool>,            // two factor auth
    pub picture: Option<String>,            // profile picture 
    pub gender: Option<String>,             // gender 
    pub bio: Option<String>,                // bio 
    pub user_account_type: Option<String>,  // admin, member, guest
    pub phone_number: Option<String>,       // phone number
    pub invite_users: Option<Vec<Uuid>>,    // invited user ids (stytch user id)
    pub referred_by: Option<String>,        // invitor user id (stytch user id)
    pub app_ids: Option<Vec<Uuid>>,         // app id arrary
    pub post_ids: Option<Vec<Uuid>>,        // post id arrary
    pub workspace_ids: Option<Vec<Uuid>>,   // workspace id arrary
    pub organization: Option<Vec<Uuid>>,    // company name
    pub latitude: Option<f64>,              // last known location coordinates
    pub longitude: Option<f64>,             
    pub last_login_ip: Option<String>,      // last login IP    
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserQuery {
    pub user_id: String,
}