use crate::model::file::AmFile;
use crate::types::FileStruct;
use futures::TryStreamExt;
use sqlx::PgPool;
use axum::extract::Extension;
use std::sync::Arc;
use axum_macros::debug_handler;

#[debug_handler]
pub async fn get_file_info(file_id: String, pool: &Arc<PgPool>) -> Result<AmFile, sqlx::Error> {
    let file_id: i32 = file_id.parse().unwrap();
    let file = sqlx::query_as!(AmFile, r#"SELECT * FROM files WHERE id = $1"#, file_id)
        .fetch_one(pool)
        .await?;

    Ok(file)
}

pub async fn _create_new_file(
    pool: &PgPool,
    user_id: String,
    file_name: String,
    file_path: String,
    file_size: i32,
) -> Result<i32, sqlx::Error> {
    let pid = 1;
    let query_res = sqlx::query_as(
        "INSERT INTO files(pid, user_id, name, path, size) VALUES ((1), (2), (3), (4), (5)) RETURNING id;")
        .bind(pid)
        .bind(user_id)
        .bind(file_name)
        .bind(file_path)
        .bind(file_size).
        fetch_one(pool).await?;

    let row: (i32,) = query_res;
    Ok(row.0)
}

pub async fn _get_root_directory_id(pool: &PgPool, user_id: String) -> Result<i32, sqlx::Error> {
    let row: (i32,) = sqlx::query_as("SELECT name FROM files WHERE id = (1)")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn _get_sub_directories(
    pool: &PgPool,
    parent_folder_id: i32,
) -> Result<i32, sqlx::Error> {
    let str_query = format!(
        "SELECT * FROM files WHERE pid = {} AND deleted = 0",
        parent_folder_id
    );
    let mut rows = sqlx::query(&str_query).fetch(pool);

    while let Some(_) = rows.try_next().await? {
        // println!("{}", row.0);
    }

    Ok(0)
}
