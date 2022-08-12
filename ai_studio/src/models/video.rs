use chrono::NaiveDate;
use schemars::schema::Schema;
use schemars::schema_for_value;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateVideoInstance {
    pub folder_id: Uuid, // folder id
    pub name: String,    // instance name
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateVideoinstance {
    pub id: Uuid,
    pub name: Option<String>,
    pub video_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub audio_batch_id: Option<Uuid>,
    #[serde(default)]
    pub image_column_id: Option<i64>,
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoInstance {
    pub id: Uuid,
    pub name: String,
    pub user_id: String,
    pub folder_id: Uuid,
    pub video_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub audio_batch_id: Option<Uuid>,
    pub image_column_id: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Video {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub url: String,
    pub length: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratedVideo {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub audio_lables: Vec<String>,
    pub name: String,
    pub user_id: String,
    pub video_instance_id: Uuid,
    pub video_url: Option<String>,
    pub vimeo_url: Option<String>,
    pub thumbnail: Option<String>,
    pub status: String,
    pub vimeo_status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for VideoInstance {
    fn default() -> Self {
        VideoInstance {
            id: Default::default(),
            name: Default::default(),
            user_id: Default::default(),
            folder_id: Default::default(),
            video_id: Default::default(),
            actor_id: Default::default(),
            audio_batch_id: Default::default(),
            image_column_id: Default::default(),
            created_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
            updated_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
        }
    }
}

impl Default for Video {
    fn default() -> Self {
        Video {
            id: Default::default(),
            user_id: Default::default(),
            name: Default::default(),
            url: Default::default(),
            length: Default::default(),
            created_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
            updated_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
        }
    }
}

impl Default for GeneratedVideo {
    fn default() -> Self {
        GeneratedVideo {
            id: Default::default(),
            batch_id: Default::default(),
            audio_lables: Default::default(),
            name: Default::default(),
            user_id: Default::default(),
            video_instance_id: Default::default(),
            video_url: Default::default(),
            vimeo_url: Default::default(),
            thumbnail: Default::default(),
            status: Default::default(),
            vimeo_status: Default::default(),
            created_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
            updated_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
        }
    }
}

impl JsonSchema for GeneratedVideo {
    fn schema_name() -> String {
        "GeneratedVideo".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(GeneratedVideo::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for Video {
    fn schema_name() -> String {
        "Video".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(Video::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for VideoInstance {
    fn schema_name() -> String {
        "VideoInstance".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(VideoInstance::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for CreateVideoInstance {
    fn schema_name() -> String {
        "CreateVideoInstance".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(CreateVideoInstance::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for UpdateVideoinstance {
    fn schema_name() -> String {
        "UpdateVideoinstance".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(UpdateVideoinstance::default());
        Schema::Object(root_schema.schema)
    }
}
