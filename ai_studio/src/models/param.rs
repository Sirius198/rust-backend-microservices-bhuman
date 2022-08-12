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
pub struct RequiredId {
    pub id: Uuid,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[query]
pub struct OptionalId {
    pub id: Option<Uuid>,
}

impl JsonSchema for RequiredId {
    fn schema_name() -> String {
        "RequiredId".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(RequiredId::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for OptionalId {
    fn schema_name() -> String {
        "OptionalId".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(OptionalId::default());
        Schema::Object(root_schema.schema)
    }
}