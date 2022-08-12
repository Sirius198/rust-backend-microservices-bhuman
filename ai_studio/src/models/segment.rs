use chrono::NaiveDate;
use openapi_rs::openapi_proc_macro::query;
use schemars::JsonSchema;
use schemars::schema::Schema;
use schemars::schema_for_value;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::chrono::NaiveDateTime;
use uuid::Uuid;

use okapi::openapi3::Parameter;
use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateSegment {
    pub video_instance_id: Uuid,
    pub prefix_time_marker_start: String,
    pub prefix_time_marker_end: String,
    pub suffix_time_marker_start: String,
    pub suffix_time_marker_end: String,
    pub audio_variable_column_id: i64,
    pub audio_variable_name: String,
    pub variable_time_marker_start: String,
    pub variable_time_marker_end: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[query]
pub struct SegmentOptionalId {
    pub id: Option<Uuid>,
    pub video_instance_id: Option<Uuid>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateSegment {
    pub id: Uuid,
    pub audio_variable_name: String,
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub id: Uuid,
    pub user_id: String,
    pub video_instance_id: Uuid,
    pub prefix_time_marker_start: String,
    pub prefix_time_marker_end: String,
    pub suffix_time_marker_start: String,
    pub suffix_time_marker_end: String,
    pub audio_variable_column_id: i64,
    pub audio_variable_name: String,
    pub variable_time_marker_start: String,
    pub variable_time_marker_end: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for Segment {
    fn default() -> Self {
        Segment {
            id: Default::default(),
            user_id: Default::default(),
            video_instance_id: Default::default(),
            prefix_time_marker_start: Default::default(),
            prefix_time_marker_end: Default::default(),
            suffix_time_marker_start: Default::default(),
            suffix_time_marker_end: Default::default(),
            audio_variable_column_id: Default::default(),
            audio_variable_name: Default::default(),
            variable_time_marker_start: Default::default(),
            variable_time_marker_end: Default::default(),
            created_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
            updated_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
        }
    }
}

impl JsonSchema for Segment {
    fn schema_name() -> String {
        "Segment".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(Segment::default());
        Schema::Object(root_schema.schema)
    }
}


impl JsonSchema for UpdateSegment {
    fn schema_name() -> String {
        "UpdateSegment".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(UpdateSegment::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for CreateSegment {
    fn schema_name() -> String {
        "CreateSegment".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(CreateSegment::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for SegmentOptionalId {
    fn schema_name() -> String {
        "SegmentOptionalId".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(SegmentOptionalId::default());
        Schema::Object(root_schema.schema)
    }
}
