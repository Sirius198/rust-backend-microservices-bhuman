use schemars::schema::Schema;
use schemars::schema_for_value;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::chrono::{NaiveDate, NaiveDateTime};
use uuid::Uuid;

#[derive(Default, Debug, Clone, PartialEq, Serialize, JsonSchema, Deserialize)]
pub struct CreateWorkspace {
    pub name: String,                // workspace name
    pub role: String,                // workspace role, owner, editor, viewer, guest
    pub description: Option<String>, // workspace description
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateWorkspace {
    pub id: Uuid,                    // workspace id (it will use for update)
    pub name: String,                // workspace name
    pub role: String,                // change role
    pub description: Option<String>, // workspace description
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddToWorkspace {
    pub id: Uuid,        // workspace id
    pub peer_id: String, // peer id (stytch user id)
    pub role: String,    // change role
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveFromWorkspace {
    pub id: Uuid,        // workspace id
    pub peer_id: String, // peer id (stytch user id)
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: i32,
    pub workspace_id: Uuid,          // workspace id
    pub user_id: String,             // user id (stytch user id)
    pub name: String,                // workspace name
    pub role: String,                // workspace role, owner, editor, viewer, guest
    pub description: Option<String>, // workspace description
    pub created_at: NaiveDateTime,   // created date
    pub updated_at: NaiveDateTime,   // updated date
}

impl JsonSchema for RemoveFromWorkspace {
    fn schema_name() -> String {
        "RemoveFromWorkspace".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(RemoveFromWorkspace::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for AddToWorkspace {
    fn schema_name() -> String {
        "AddToWorkspace".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(AddToWorkspace::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for UpdateWorkspace {
    fn schema_name() -> String {
        "UpdateWorkspace".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(UpdateWorkspace::default());
        Schema::Object(root_schema.schema)
    }
}

impl JsonSchema for Workspace {
    fn schema_name() -> String {
        "Workspace".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let root_schema = schema_for_value!(Workspace {
            created_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
            updated_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
            id: 0,
            workspace_id: Uuid::new_v4(),
            user_id: String::new(),
            name: String::new(),
            role: String::new(),
            description: Some(String::new())
        });
        Schema::Object(root_schema.schema)
    }
}
