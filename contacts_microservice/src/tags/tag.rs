use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;
use schemars::JsonSchema;
use schemars::schema::Schema;
use schemars::schema_for_value;


#[derive(Default, Debug, Clone, PartialEq, JsonSchema, Serialize, Deserialize)]
pub struct CreateTag {
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateTag {
    pub id: Uuid,
    pub name: String,
}

impl JsonSchema for UpdateTag {
    fn schema_name() -> String {
        "UpdateTag".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(UpdateTag::default());
        Schema::Object(root_schema.schema)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct TagInfo {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
}

impl JsonSchema for TagInfo {
    fn schema_name() -> String {
        "TagInfo".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(TagInfo::default());
        Schema::Object(root_schema.schema)
    }
}