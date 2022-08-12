use schemars::JsonSchema;
use schemars::schema::Schema;
use schemars::schema_for_value;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::chrono::NaiveDate;
use sqlx::types::chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, JsonSchema, Serialize, Deserialize)]
pub struct CreateUser {
    pub first_name: String,           // first name
    pub last_name: String,            // last name
    pub email: Option<String>,        // email
    pub phone_number: Option<String>, // phone number
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateUser {
    pub first_name: Option<String>,        // first name
    pub last_name: Option<String>,         // last name
    pub username: Option<String>,          // username
    pub email: Option<String>,             // email
    pub dob: Option<NaiveDateTime>,        // birthday
    pub two_fator: Option<bool>,           // two factor auth
    pub picture: Option<String>,           // profile picture
    pub gender: Option<String>,            // gender
    pub bio: Option<String>,               // bio
    pub user_account_type: Option<String>, // admin, member, guest
    pub phone_number: Option<String>,      // phone number
    pub latitude: Option<f32>,             // last known location coordinates
    pub longitude: Option<f32>,
    pub last_login_ip: Option<String>, // last login IP
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub user_id: String,                   // user id
    pub first_name: String,                // first name
    pub last_name: String,                 // last name
    pub email: String,                     // email
    pub phone_number: String,              // phone number
    pub last_at: NaiveDateTime,            // last login date
    pub username: Option<String>,          // username
    pub dob: Option<NaiveDateTime>,        // birthday
    pub two_fator: Option<bool>,           // two factor auth
    pub picture: Option<String>,           // profile picture
    pub gender: Option<String>,            // gender
    pub bio: Option<String>,               // bio
    pub user_account_type: Option<String>, // admin, member, guest
    pub invite_users: Option<Vec<Uuid>>,   // invited user ids (stytch user id)
    pub referred_by: Option<String>,       // invitor user id (stytch user id)
    pub app_ids: Option<Vec<Uuid>>,        // app id arrary
    pub post_ids: Option<Vec<Uuid>>,       // post id arrary
    pub workspace_ids: Option<Vec<Uuid>>,  // workspace id arrary
    pub organization: Option<Vec<Uuid>>,   // company name
    pub latitude: Option<f32>,             // last known location coordinates
    pub longitude: Option<f32>,
    pub last_login_ip: Option<String>, // last login IP
}

impl Default for UpdateUser {
    fn default() -> Self {
        UpdateUser {
            first_name: Some(String::new()),
            last_name: Some(String::new()),
            username: Some(String::new()),
            email: Some(String::new()),
            dob: Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11)),
            two_fator: Some(true),
            picture: Some(String::new()),
            gender: Some(String::new()),
            bio: Some(String::new()),
            user_account_type: Some(String::new()),
            phone_number: Some(String::new()),
            latitude: Some(0.0),
            longitude: Some(0.0),
            last_login_ip: Some(String::new()),
        }
    }
}



impl Default for User {
    fn default() -> Self {
        User {
            id: Uuid::new_v4(),
            user_id: String::new(),
            first_name: String::new(),
            last_name: String::new(),
            email: String::new(),
            phone_number: String::new(),
            last_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
            username: Some(String::new()),
            dob: Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11)),
            two_fator: Some(false),
            picture: Some(String::new()),
            gender: Some(String::new()),
            bio: Some(String::new()),
            user_account_type: Some(String::new()),
            invite_users: Some(Vec::new()),
            referred_by: Some(String::new()),
            app_ids: Some(Vec::new()),
            post_ids: Some(Vec::new()),
            workspace_ids: Some(Vec::new()),
            organization: Some(Vec::new()),
            latitude: Some(0.0),
            longitude: Some(0.0),
            last_login_ip: Some(String::new()),
        }
    }
}


impl JsonSchema for UpdateUser {
    fn schema_name() -> String {
        "UpdateUser".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(UpdateUser::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for User {
    fn schema_name() -> String {
        "User".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(User::default());
        Schema::Object(root_schema.schema)
    }
}