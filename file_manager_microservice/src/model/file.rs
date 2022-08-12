use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use sqlx::types::chrono::NaiveDateTime;
use schemars::schema::Schema;
use schemars::schema_for_value;
use schemars::JsonSchema;

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmFile {
    pub id: i32,
    pub pid: i32,
    pub user_id: String,
    pub name: String,
    pub path: String,
    pub size: i32,
    pub status: i32,
    pub deleted: i32,
    pub is_folder: i32,
    pub created_at: NaiveDateTime, // create date
    pub updated_at: NaiveDateTime, // update date
}

impl Default for AmFile {
    fn default() -> Self {
        Self {
            id: Default::default(),
            pid: 0,
            user_id: String::new(),
            name: String::new(),
            path: String::new(),
            size: 0,
            status: 0,
            is_folder: 0,
            deleted: 0,
            created_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
            updated_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
        }
    }
}

impl JsonSchema for AmFile {
    fn schema_name() -> String {
        "AmFile".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(AmFile::default());
        Schema::Object(root_schema.schema)
    }
}