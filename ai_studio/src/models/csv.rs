use openapi_rs::openapi_proc_macro::query;
use schemars::JsonSchema;
use schemars::schema::Schema;
use schemars::schema_for_value;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use okapi::openapi3::Parameter;
use okapi::openapi3::RefOr;
use openapi_rs::gen::OpenApiGenerator;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[query]
pub struct CsvRequiredId {
    pub video_instance_id: Uuid,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[query]
pub struct AudioBatchId {
    pub audio_batch_id: Uuid,
}

impl JsonSchema for CsvRequiredId {
    fn schema_name() -> String {
        "CsvRequiredId".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(CsvRequiredId::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for AudioBatchId {
    fn schema_name() -> String {
        "AudioBatchId".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(AudioBatchId::default());
        Schema::Object(root_schema.schema)
    }
}