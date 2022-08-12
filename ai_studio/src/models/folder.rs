use chrono::NaiveDate;
use openapi_rs::openapi_proc_macro::query;
use schemars::schema::Schema;
use schemars::schema_for_value;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::chrono::NaiveDateTime;
use uuid::Uuid;

use okapi::openapi3::Parameter;
use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFolder {
    pub workspace_id: Uuid, // workspace id
    pub name: String,       // folder name
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFolder {
    pub id: Uuid,     // folder id
    pub name: String, // folder name
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[query]
pub struct FolderOptionalId {
    pub id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Folder {
    pub id: Uuid,                  // folder id
    pub user_id: String,           // user id
    pub workspace_id: Uuid,        // workspace id
    pub name: String,              // folder name
    pub parent_videos: i64,        // parent videos
    pub generated_videos: i64,     // generated videos
    pub created_at: NaiveDateTime, // create date
    pub updated_at: NaiveDateTime, // update date
}

impl Default for Folder {
    fn default() -> Self {
        Self {
            id: Default::default(),
            user_id: Default::default(),
            workspace_id: Default::default(),
            name: Default::default(),
            parent_videos: Default::default(),
            generated_videos: Default::default(),
            created_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
            updated_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
        }
    }
}

impl JsonSchema for Folder {
    fn schema_name() -> String {
        "Folder".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(Folder::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for FolderOptionalId {
    fn schema_name() -> String {
        "FolderOptionalId".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(FolderOptionalId::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for CreateFolder {
    fn schema_name() -> String {
        "CreateFolder".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(CreateFolder::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for UpdateFolder {
    fn schema_name() -> String {
        "UpdateFolder".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(UpdateFolder::default());
        Schema::Object(root_schema.schema)
    }
}
