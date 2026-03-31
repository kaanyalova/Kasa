use sqlx::{Pool, QueryBuilder, Sqlite, query, query_scalar};

use super::indexer::index;
/// Adds a single index source from the path, does not index that path without calling index_path()
pub async fn add_index_source_impl(path: &str, pool: &Pool<Sqlite>) {
    query("INSERT INTO IndexSource(path) VALUES (?)")
        .bind(path)
        .execute(pool)
        .await
        .unwrap();
}

/// Removes files that are imported from the given path and marks any files without paths any references to paths
pub async fn remove_index_source_impl(path: &str, pool: &Pool<Sqlite>) {
    query("DELETE FROM IndexSource WHERE path = ?")
        .bind(path)
        .execute(pool)
        .await
        .unwrap();

    query("DELETE FROM Path WHERE imported_from = ?")
        .bind(path)
        .execute(pool)
        .await
        .unwrap();

    // Mark any unreferenced files
    query("UPDATE Media SET has_file_ref = false WHERE NOT EXISTS (SELECT 1 FROM Path WHERE Path.hash = Media.hash)").execute(pool).await.unwrap();
}

/// Indexes all paths stored in the db
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

/// Gets all indexed paths stored in the db
pub async fn get_index_paths_impl(pool: &Pool<Sqlite>) -> Vec<String> {
    let paths: Vec<String> = query_scalar("SELECT * FROM IndexSource")
        .fetch_all(pool)
        .await
        .unwrap();
    paths
}

/// Removes all the data of all the media which contain no path references
pub async fn cleanup_unreferenced_files_impl(pool: &Pool<Sqlite>, pool_thumbs: &Pool<Sqlite>) {
    query("DELETE FROM HashTagPair WHERE HashTagPair.hash IN (SELECT Media.hash FROM Media WHERE Media.has_file_ref = false)").execute(pool).await.unwrap();
    query("DELETE FROM Image WHERE Image.hash IN (SELECT Media.hash FROM Media WHERE Media.has_file_ref = false)").execute(pool).await.unwrap();
    query("DELETE FROM MediaGroupEntry WHERE MediaGroupEntry.hash IN (SELECT Media.hash FROM Media WHERE Media.has_file_ref = false)").execute(pool).await.unwrap();
    query("DELETE FROM Media WHERE has_file_ref = false")
        .execute(pool)
        .await
        .unwrap();

    let hashes_to_delete: Vec<String> =
        query_scalar("SELECT Media.hash FROM Media WHERE Media.has_file_ref = false")
            .fetch_all(pool)
            .await
            .unwrap();

    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("DELETE FROM Thumbs WHERE hash IN (");

    let mut separated = query_builder.separated(", ");
    for hash in hashes_to_delete {
        separated.push_bind(hash);
    }

    separated.push_unseparated(") ");

    let delete_query = query_builder.build();
    delete_query.execute(pool_thumbs).await.unwrap();
}

/// Removes all the data for the media
pub async fn nuke_selected_index_impl(
    pool: &Pool<Sqlite>,
    pool_thumbs: Option<&Pool<Sqlite>>,
    path: &str,
) {
    query("DELETE FROM Media WHERE Media.hash IN (SELECT Path.hash FROM Path WHERE imported_from = ? GROUP BY Path.path HAVING COUNT(*) =1)")
        .bind(path)
        .execute(pool)
        .await
        .unwrap();

    // delete all the tags from the selected items
    query("DELETE FROM HashTagPair WHERE HashTagPair.hash IN (SELECT Path.hash FROM Path WHERE imported_from = ? GROUP BY Path.path HAVING COUNT(*) =1)")
        .bind(path)
        .execute(pool)
        .await
        .unwrap();

    // delete all the image data from the selected items
    query("DELETE FROM Image WHERE Image.hash IN (SELECT Path.hash FROM Path WHERE Path.imported_from = ? GROUP BY Path.path HAVING COUNT(*) =1)")
        .bind(path)
        .execute(pool)
        .await
        .unwrap();

    // delete any group entries
    query("DELETE FROM MediaGroupEntry WHERE MediaGroupEntry.hash IN (SELECT Path.hash FROM Path WHERE Path.imported_from = ? GROUP BY Path.path HAVING COUNT(*) =1)")
        .bind(path)
        .execute(pool)
        .await
        .unwrap();

    // delete thumbnails if they exist
    if let Some(pool_thumbs) = pool_thumbs {
        let hashes_to_delete: Vec<String> =
            query_scalar("SELECT hash FROM Path WHERE Path.imported_from = ? GROUP BY Path.path HAVING COUNT(*) = 1")
                .bind(path)
                .fetch_all(pool)
                .await
                .unwrap();

        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("DELETE FROM Thumbs WHERE hash IN (");

        let mut separated = query_builder.separated(", ");
        for hash in hashes_to_delete {
            separated.push_bind(hash);
        }

        separated.push_unseparated(") ");

        let delete_query = query_builder.build();
        delete_query.execute(pool_thumbs).await.unwrap();
    }

    remove_index_source_impl(path, pool).await;

    // This should be the last one
    query("DELETE FROM Path WHERE imported_from = ?")
        .bind(path)
        .execute(pool)
        .await
        .unwrap();

    //if let Some(pool_thumbs) = pool_thumbs {
    //    delete_entries("Thumbs", &hashes_to_delete, pool_thumbs).await
    //}
}

pub async fn nuke_all_indexes_impl(pool: &Pool<Sqlite>, pool_thumbs: Option<&Pool<Sqlite>>) {
    let paths = get_index_paths_impl(pool).await;

    for path in paths {
        nuke_selected_index_impl(pool, pool_thumbs, &path).await;
    }
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
