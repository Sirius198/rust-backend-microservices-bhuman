use crate::model::file::AmFile;
use axum::{
    extract::{Extension, Path},
    // response::IntoResponse,
    Json,
};
// use axum_macros::debug_handler;
// use microservice_utils::server::response::{into_response, AxumRes, AxumResult};
use microservice_utils::{
    jwt::extractor::AuthToken,
    server::response::{into_response, AxumRes, AxumResult},
};
use sqlx::postgres::PgPool;
use std::sync::Arc;

/*pub fn get_sub_directory() {}
pub fn create_new_directory() {}
pub fn move_folder() {}
pub fn copy_folder() {}
pub fn delete_folder() {}*/

// #[debug_handler]
async fn db_test_has_root_directory(pool: &PgPool, user_id: String) -> Result<i32, sqlx::Error> {
    let row: (i32,) = sqlx::query_as("SELECT id FROM files WHERE user_id = $1 AND pid = 0;")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok(row.0)
}

pub async fn db_get_root_directory_id(pool: &PgPool, user_id: String) -> Result<i32, sqlx::Error> {
    if let Ok(id) = db_test_has_root_directory(pool, user_id.clone()).await {
        return Ok(id);
    }

    let t = db_create_new_directory(pool, user_id, 0, "".to_owned()).await?;
    Ok(t.id)
}

pub async fn get_root_directory_id(
    Extension(pool): Extension<Arc<PgPool>>,
    AuthToken(user_id): AuthToken,
) -> AxumResult<Json<AxumRes<i32>>> {

    let t = db_get_root_directory_id(&pool, user_id).await;
    match t {
        Ok(id) => {
            return Ok(axum::Json(AxumRes {
                code: 200,
                result: id,
            }));
        }
        Err(e) => {
            let ret = serde_json::json!({
                "error": format!("{:?}", e),
            });
            return Err(into_response(400, ret));
        }
    }
}

// #[debug_handler]
pub async fn db_create_new_directory(
    pool: &PgPool,
    user_id: String,
    pid: i32,
    name: String,
) -> Result<AmFile, sqlx::Error> {
    let query_str = format!("INSERT INTO files(pid, user_id, name, path, size, status, is_folder) VALUES ({},{},\'{}\',\'{}\',{},{},{}) RETURNING *;",
        pid, user_id, name, "", 0, 0, 1);

    let file = sqlx::query_as::<_, AmFile>(&query_str)
        .fetch_one(pool)
        .await?;

    Ok(file)
}

async fn db_get_sub_directories(
    pool: &PgPool,
    user_id: String,
    parent_folder_id: i32,
) -> Result<Vec<AmFile>, sqlx::Error> {
    if parent_folder_id == 0 {
        let x = db_test_has_root_directory(pool, user_id.clone()).await;
        match x {
            Ok(_) => {
                println!("ook");
            }
            Err(e) => {
                println!("Error not found root directory: {}", e);
                if let Err(e) = db_create_new_directory(pool, user_id.clone(), 0, "".to_owned()).await {
                    println!("{}", e);
                }
            }
        }
    }

    let query_str = format!(
        "SELECT * FROM files WHERE user_id = {} AND pid = {};",
        user_id, parent_folder_id
    );
    let files = sqlx::query_as::<_, AmFile>(&query_str)
        .fetch_all(pool)
        .await?;

    Ok(files)
}

pub async fn get_sub_directory(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(folder_id): Path<String>,
    AuthToken(user_id): AuthToken,
) -> AxumResult<Json<AxumRes<Vec<AmFile>>>> {
    let parent_folder_id: i32 = folder_id.parse::<i32>().unwrap();

    let dirs = db_get_sub_directories(&pool, user_id, parent_folder_id).await;
    match dirs {
        Ok(result) => Ok(axum::Json(AxumRes {
            code: 200,
            result: result,
        })),
        Err(e) => {
            println!("{:?}", e.to_string());
            let ret = serde_json::json!({
                "error": format!("{:?}", e),
            });
            Err(into_response(500, ret))
        }
    }
}

async fn db_test_folder_permission(
    pool: &PgPool,
    user_id: String,
    folder_id: i32,
) -> Result<(i32, i32), sqlx::Error> {
    let query_str = format!(
        "SELECT id, is_folder FROM files WHERE user_id = {} AND id = {};",
        user_id, folder_id
    );
    let row: (i32, i32) = sqlx::query_as(&query_str).fetch_one(pool).await?;

    Ok((row.0, row.1))
}

async fn db_test_file_folder_permission(
    pool: &PgPool,
    user_id: String,
    ff_id: i32,
) -> Result<(i32, i32), sqlx::Error> {
    let query_str = format!(
        "SELECT id, pid FROM files WHERE user_id = {} AND id = {};",
        user_id, ff_id
    );
    let row: (i32, i32) = sqlx::query_as(&query_str).fetch_one(pool).await?;

    Ok((row.0, row.1))
}

pub async fn create_new_folder(
    Extension(pool): Extension<Arc<PgPool>>,
    Path((folder_id, folder_name)): Path<(String, String)>,
    AuthToken(user_id): AuthToken,
) -> AxumResult<Json<AxumRes<AmFile>>> {
    
    let parent_folder_id: i32 = folder_id.parse::<i32>().unwrap();

    if let Err(e) = db_test_folder_permission(&pool, user_id.clone(), parent_folder_id).await {
        println!("test err: {}", e);

        let ret = serde_json::json!({
            "error": "No permission",
        });
        return Err(into_response(400, ret));
    }

    let res = db_create_new_directory(&pool, user_id, parent_folder_id, folder_name).await;
    match res {
        Ok(folder) => Ok(axum::Json(AxumRes {
            code: 200,
            result: folder,
        })),
        Err(e) => {
            let ret = serde_json::json!({
                "error": format!("{:?}", e),
            });
            Err(into_response(400, ret))
        }
    }
}

async fn db_move_file(pool: &PgPool, file_id: i32, pid: i32) -> Result<(), sqlx::Error> {
    let query_str = format!("UPDATE files SET pid = {} WHERE id = {}", pid, file_id);
    sqlx::query(&query_str).execute(pool).await?;

    Ok(())
}

pub async fn move_folder_or_file(
    Path((src_folder_id, dst_folder_id)): Path<(String, String)>,
    Extension(pool): Extension<Arc<PgPool>>,
    AuthToken(user_id): AuthToken,
) -> AxumResult<Json<AxumRes<String>>> {
    
    let src_id: i32 = src_folder_id.parse().unwrap();
    let dst_id: i32 = dst_folder_id.parse().unwrap();

    if let Err(e) = db_test_folder_permission(&pool, user_id.clone(), src_id).await {
        let ret = serde_json::json!({
            "error": format!("{:?}", e),
        });
        return Err(into_response(400, ret));
    }

    // Check if dest is folder
    let t = db_test_folder_permission(&pool, user_id.clone(), dst_id).await;

    match t {
        Ok(file) => {
            if file.1 == 0 {
                let ret = serde_json::json!({
                    "error": "not a folder",
                });
                return Err(into_response(400, ret));
            }
        }
        Err(e) => {
            let ret = serde_json::json!({
                "error": format!("{:?}", e),
            });
            return Err(into_response(400, ret));
        }
    }
    println!("he");

    if let Err(e) = db_move_file(&pool, src_id, dst_id).await {
        let ret = serde_json::json!({
            "error": format!("{:?}", e),
        });
        return Err(into_response(400, ret));
    }

    Ok(axum::Json(AxumRes {
        code: 200,
        result: "success".to_owned(),
    }))
}

async fn db_rename_file(pool: &PgPool, file_id: i32, file_name: String) -> Result<(), sqlx::Error> {
    let query_str = format!(
        "UPDATE files SET name = \'{}\' WHERE id = {};",
        file_name, file_id
    );
    sqlx::query(&query_str).execute(pool).await?;

    Ok(())
}

pub async fn rename_folder(
    Path((file_id, file_name)): Path<(String, String)>,
    Extension(pool): Extension<Arc<PgPool>>,
    AuthToken(user_id): AuthToken,
) -> AxumResult<Json<AxumRes<String>>> {
    
    let file_id: i32 = file_id.parse().unwrap();

    if let Err(e) = db_test_file_folder_permission(&pool, user_id, file_id).await {
        let ret = serde_json::json!({
            "error": format!("{:?}", e),
        });
        return Err(into_response(400, ret));
    }

    if let Err(e) = db_rename_file(&pool, file_id, file_name).await {
        let ret = serde_json::json!({
            "error": format!("{:?}", e),
        });
        return Err(into_response(400, ret));
    }

    Ok(axum::Json(AxumRes {
        code: 200,
        result: "success".to_owned(),
    }))
}
