use chrono::NaiveDate;
use schemars::JsonSchema;
use schemars::schema::Schema;
use schemars::schema_for_value;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Audio {
    pub id: Uuid,
    pub user_id: String,
    pub actor_id: Uuid,
    pub name: String,
    pub url: String,
    pub audio_length: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(sqlx::FromRow)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioBatch {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for Audio {
    fn default() -> Self {
        Audio {
            id: Uuid::new_v4(),
            name: String::new(),
            user_id: String::new(),
            actor_id:Uuid::new_v4(),
            url: String::new(),
            audio_length: String::new(),
            created_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
            updated_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
        }
    }
}

impl Default for AudioBatch {
    fn default() -> Self {
        AudioBatch {
            id: Uuid::new_v4(),
            name: String::new(),
            user_id: String::new(),
            created_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
            updated_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
        }
    }
}

impl JsonSchema for Audio {
    fn schema_name() -> String {
        "Audio".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(Audio::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for AudioBatch {
    fn schema_name() -> String {
        "AudioBatch".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(AudioBatch::default());
        Schema::Object(root_schema.schema)
    }
}