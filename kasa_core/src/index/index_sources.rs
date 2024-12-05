use sqlx::{
    pool, query, query_as, query_builder, query_scalar, Pool, QueryBuilder, Sqlite, SqlitePool,
};

use crate::{
    db::{migrations::prepare_dbs, schema::Media},
    test_util::db_utils::{_insert_media_row, insert_media_row, insert_path_row},
};

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

pub async fn nuke_selected_index(
    pool: &Pool<Sqlite>,
    pool_thumbs: Option<&Pool<Sqlite>>,
    path: &str,
) {
    query("DELETE FROM Path WHERE imported_from = ?")
        .bind(path)
        .execute(pool)
        .await
        .unwrap();

    // get all hashes to delete where there is no other references left in the path table
    // sqlite doesn't support RETUNING clauses making me suffer                                                                                                                              ccc              c
    //let hashes_to_delete: Vec<String> = query_scalar("FUCK FUCK FUCK FUCK FUCK FUCK FUCK")
    //    .bind(path)
    //    .bind(path)
    //    .fetch_all(pool)
    //    .await
    //    .unwrap();

    //delete_entries("Media", &hashes_to_delete, pool).await;
    //delete_entries("HashTagPair", &hashes_to_delete, pool).await;
    //delete_entries("Image", &hashes_to_delete, pool).await;

    //if let Some(pool_thumbs) = pool_thumbs {
    //    delete_entries("Thumbs", &hashes_to_delete, pool_thumbs).await
    //}
}
async fn delete_entries(table: &str, hashes_to_delete: &Vec<String>, pool: &Pool<Sqlite>) {
    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new(format!("DELETE FROM {} WHERE hash IN (", table));
    let mut separated = query_builder.separated(",");
    for hash in hashes_to_delete {
        separated.push_bind(hash);
    }
    separated.push_bind_unseparated(")");

    let q = query_builder.build();

    q.execute(pool).await.unwrap();
}

/*
#[sqlx::test]
fn test_nuke_selected(pool: SqlitePool) {
    sqlx::migrate!("../migrations/db").run(&pool).await.unwrap();

    insert_media_row(&pool, "hash1", "", "", 0, "", 0, 0, 0, false, false).await;
    insert_media_row(&pool, "hash2", "", "", 0, "", 0, 0, 0, false, false).await;
    insert_media_row(&pool, "hash3", "", "", 0, "", 0, 0, 0, false, false).await;

    insert_path_row(&pool, "hash1", "a", "path1").await;
    insert_path_row(&pool, "hash1", "b", "path2").await;
    insert_path_row(&pool, "hash1", "c", "path3").await;

    insert_path_row(&pool, "hash2", "", "path1").await;

    nuke_selected_index(&pool, None, "path1").await;
    nuke_selected_index(&pool, None, "path3").await;

    let media: Vec<Media> = query_as("SELECT * FROM Media")
        .fetch_all(&pool)
        .await
        .unwrap();

    dbg!(media);
}
*/
