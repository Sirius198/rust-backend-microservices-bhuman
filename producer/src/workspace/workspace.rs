use serde::Deserialize;
use serde::Serialize;
use sqlx::types::chrono::NaiveDateTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateWorkspace {
    pub user_id: String,                    // user id (stytch user id)
    pub name: String,                       // workspace name
    pub description: Option<String>,        // workspace description
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateWorkspace {
    pub id: i32,                            // workspace id (it will use for update)
    pub user_id: String,                    // user id (stytch user id)
    pub name: String,                       // workspace name
    pub description: Option<String>,        // workspace description
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceQuery {
    pub user_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: i32,                            // workspace id
    pub user_id: String,                    // user id (stytch user id)
    pub name: String,                       // workspace name
    pub description: Option<String>,        // workspace description
    pub created_at: NaiveDateTime,          // last login date
    pub updated_at: NaiveDateTime,          // last login date
}