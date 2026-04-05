use std::path::{Path, PathBuf};

use kasa_ai::image_embeddings::generate_image_embeddings;
use kasa_core::{
    config::global_config::get_config_impl,
    db::embeddings::{EmbeddingResult, insert_embeddings},
};
use sqlx::{query_as, sqlite::SqlitePoolOptions};

use crate::ai_tag_images::HashAndPath;

pub async fn generate_all_image_embeddings() {
    let config = get_config_impl();

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&config.db.db_path)
        .await
        .unwrap();

    println!("connected to the db at {}", &config.db.db_path);

    let hashes_and_paths: Vec<HashAndPath> = query_as(
        "SELECT m.hash, MIN(p.path) as path FROM Media m JOIN Path p ON p.hash = m.hash WHERE m.media_type = 'Image' GROUP BY m.hash",
    )
        .fetch_all(&pool)
        .await
        .unwrap();

    let hashes_and_paths: Vec<HashAndPath> = hashes_and_paths
        .into_iter()
        .filter(|hash_and_path| PathBuf::from(&hash_and_path.path).exists())
        .collect();

    println!("{} Hashes found", hashes_and_paths.len());

    let path_refs: Vec<&Path> = hashes_and_paths
        .iter()
        .map(|p| Path::new(&p.path))
        .collect();

    let embeddings = generate_image_embeddings(path_refs).unwrap();

    let embeddings: Vec<EmbeddingResult> = embeddings
        .into_iter()
        .enumerate()
        .map(|(i, result)| EmbeddingResult {
            hash: hashes_and_paths[i].hash.clone(),
            embedding: result,
        })
        .collect();

    insert_embeddings(&pool, embeddings).await.unwrap();
}
