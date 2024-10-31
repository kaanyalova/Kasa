use sqlx::{pool, query, query_scalar, Pool, Sqlite};

use super::indexer::index;

pub async fn add_index_source_impl(path: &str, pool: &Pool<Sqlite>) {
    query("INSERT INTO IndexSource(path) VALUES (?)")
        .bind(&path)
        .execute(pool)
        .await
        .unwrap();
}

pub async fn index_single_source_impl(
    path: String,
    pool: &Pool<Sqlite>,
    pool_thumbs: &Pool<Sqlite>,
) {
    index(&path, pool, pool_thumbs).await;
}

pub async fn remove_index_source_impl(path: &str, pool: &Pool<Sqlite>) {
    query("DELETE FROM IndexSource WHERE path = ?")
        .bind(&path)
        .execute(pool)
        .await
        .unwrap();
}

pub async fn index_all_impl(pool: &Pool<Sqlite>, pool_thumbs: &Pool<Sqlite>) {
    let paths: Vec<String> = query_scalar("SELECT * FROM IndexSource")
        .fetch_all(pool)
        .await
        .unwrap();

    // not parallelized as indexing is parallel anyways
    for path in paths {
        index(&path, pool, pool_thumbs).await;
    }
}

pub async fn get_index_paths_impl(pool: &Pool<Sqlite>) -> Vec<String> {
    let paths: Vec<String> = query_scalar("SELECT * FROM IndexSource")
        .fetch_all(pool)
        .await
        .unwrap();
    return paths;
}

pub async fn index_single_impl() {
    todo!()
}
