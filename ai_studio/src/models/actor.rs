use chrono::NaiveDate;
use schemars::schema::Schema;
use schemars::schema_for_value;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, JsonSchema, Serialize, Deserialize)]
pub struct CreateActor {
    pub name: String, // actor name
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateActor {
    pub id: Uuid,     // actor id
    pub name: String, // actor name
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Actor {
    pub id: Uuid,                  // folder id
    pub user_id: String,           // user id
    pub name: String,              // actor name
    pub created_at: NaiveDateTime, // create date
    pub updated_at: NaiveDateTime, // update date
}

impl Default for UpdateActor {
    fn default() -> Self {
        UpdateActor {
            id: Uuid::new_v4(),
            name: String::new(),
        }
    }
}

impl Default for Actor {
    fn default() -> Self {
        Actor {
            id: Uuid::new_v4(),
            name: String::new(),
            user_id: String::new(),
            created_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
            updated_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
        }
    }
}

impl JsonSchema for Actor {
    fn schema_name() -> String {
        "Actor".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(Actor::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for UpdateActor {
    fn schema_name() -> String {
        "UpdateActor".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(UpdateActor::default());
        Schema::Object(root_schema.schema)
    }
}
