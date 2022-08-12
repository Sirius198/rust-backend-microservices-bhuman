use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;
use schemars::JsonSchema;
use schemars::schema::Schema;
use schemars::schema_for_value;

use crate::contacts::contacts::GenericContact;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagPeople {
    pub tag_id: Uuid,
    pub identifier: String,
}

impl JsonSchema for TagPeople {
    fn schema_name() -> String {
        "TagPeople".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(TagPeople::default());
        Schema::Object(root_schema.schema)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagPeopleResult {
    pub user_id: String,
    pub tag_id: Uuid,
    pub contacts: Vec<GenericContact>,
}

impl JsonSchema for TagPeopleResult {
    fn schema_name() -> String {
        "TagPeopleResult".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(TagPeopleResult::default());
        Schema::Object(root_schema.schema)
    }
}