use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use image::DynamicImage;
use itertools::Itertools;
use kasa_ai::image_embeddings::{
    Embedding, generate_image_embeddings, generate_image_embeddings_from_images,
};
use kasa_core::{
    config::global_config::get_config_impl,
    db::{
        embeddings::{self, EmbeddingResult, insert_embeddings},
        migrations::prepare_dbs,
    },
    thumbnail::{extract_frame, get_buffer},
};
use libsqlite3_sys::sqlite3_auto_extension;
use sqlite_vec::sqlite3_vec_init;
use sqlx::{SqlitePool, query_as, sqlite::SqlitePoolOptions};
use tokio::{sync::mpsc, task::spawn_blocking};

use crate::ai_tag_images::HashAndPath;

const EMBEDDING_CHUNK_SIZE: usize = 100;

pub async fn generate_all_image_embeddings() {
    let config = get_config_impl();
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }

    prepare_dbs(&config).await;

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&config.db.db_path)
        .await
        .unwrap();

    let hashes_and_paths: Vec<HashAndPath> = query_as(
        "SELECT m.hash, p.path as path FROM Media m JOIN Path p ON p.hash = m.hash LEFT JOIN MediaEmbeddingMeta mem ON mem.hash = m.hash WHERE m.media_type = 'Image' AND mem.hash IS NULL",
    )
        .fetch_all(&pool)
        .await
        .unwrap();

    let mut hashes_and_paths: Vec<HashAndPath> = hashes_and_paths
        .into_iter()
        .filter(|hash_and_path| PathBuf::from(&hash_and_path.path).exists())
        .collect();

    hashes_and_paths.sort_by(|a, b| a.hash.cmp(&b.hash));

    let first_valid_hashes_and_paths: Vec<HashAndPath> = hashes_and_paths
        .chunk_by(|a, b| a.hash == b.hash)
        .filter_map(|hnps| {
            hnps.into_iter()
                .find(|hnp| PathBuf::from(&hnp.path).exists())
                .cloned()
        })
        .collect();

    let hash_lookup: HashMap<PathBuf, String> = first_valid_hashes_and_paths
        .iter()
        .map(|hp| (PathBuf::from(&hp.path), hp.hash.clone()))
        .collect();

    let total_hashes = first_valid_hashes_and_paths.len();
    println!("{} Hashes found", total_hashes);

    let (tx, mut rx) = mpsc::channel(10);

    spawn_blocking(move || {
        first_valid_hashes_and_paths
            .clone()
            .iter()
            .map(|p| Path::new(&p.path))
            .chunks(EMBEDDING_CHUNK_SIZE)
            .into_iter()
            .enumerate()
            .filter_map(|(i, p)| {
                let collected: Vec<&Path> = p.collect();
                let result = generate_image_embeddings(collected).ok();

                println!(
                    "Created embeddings for {}/{}",
                    (i + 1) * EMBEDDING_CHUNK_SIZE,
                    total_hashes
                );

                result
            })
            .for_each(|chunk| {
                let db_batch: Vec<EmbeddingResult> = chunk
                    .into_iter()
                    .filter_map(|e| {
                        let hash = hash_lookup.get(&e.path)?;
                        Some(EmbeddingResult {
                            hash: hash.clone(),
                            embedding: e.params,
                        })
                    })
                    .collect();

                if !db_batch.is_empty() {
                    let _ = tx.blocking_send(db_batch);
                }
            });
    });

    while let Some(batch) = rx.recv().await {
        insert_embeddings(&pool, batch).await.unwrap();
    }

    let mut videos:Vec<HashAndPath> = query_as(
        "SELECT m.hash, p.path as path FROM Media m JOIN Path p ON p.hash = m.hash LEFT JOIN MediaEmbeddingMeta mem ON mem.hash = m.hash WHERE m.media_type = 'Video' AND mem.hash IS NULL",
    )
        .fetch_all(&pool)
        .await
        .unwrap();
    videos.sort_by(|a, b| a.hash.cmp(&b.hash));
    let valid_videos: Vec<HashAndPath> = videos
        .chunk_by(|a, b| a.hash == b.hash)
        .filter_map(|hnps| {
            hnps.into_iter()
                .find(|hnp| PathBuf::from(&hnp.path).exists())
                .cloned()
        })
        .collect();

    // todo make these into chunks as well
    let frames: Vec<(String, DynamicImage)> = valid_videos
        .into_iter()
        .filter_map(|v| {
            let frame = match extract_frame(&v.path, 5000) {
                Ok(f) => f,
                Err(e) => {
                    println!("Error extracting frame for {}: {:?}", v.path, e);
                    return None;
                }
            };
            let buffer = match get_buffer(&frame.0) {
                Ok(b) => b,
                Err(e) => {
                    println!("Error getting buffer for {}: {:?}", v.path, e);
                    return None;
                }
            };
            Some((v.hash.clone(), DynamicImage::from(buffer)))
        })
        .collect();

    println!("extracted {} videos", frames.len());

    for (i, chunk) in frames.chunks(EMBEDDING_CHUNK_SIZE).enumerate() {
        let (hashes, images): (Vec<String>, Vec<DynamicImage>) = chunk.iter().cloned().unzip();
        if let Ok(embeddings) = generate_image_embeddings_from_images(images) {
            let db_batch: Vec<EmbeddingResult> = hashes
                .into_iter()
                .zip(embeddings)
                .map(|(hash, embedding)| EmbeddingResult { hash, embedding })
                .collect();

            println!(
                "Created video embeddings for {}/{}",
                (i * EMBEDDING_CHUNK_SIZE) + chunk.len(),
                frames.len()
            );

            if !db_batch.is_empty() {
                insert_embeddings(&pool, db_batch).await.unwrap();
            }
        }
    }
}
