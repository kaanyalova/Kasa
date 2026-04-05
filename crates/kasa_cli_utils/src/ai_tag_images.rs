use std::{
    env,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use kasa_ai::{
    Session, prepare_session,
    wdv_tagger::{TaggerThresholds, prepare_labels, tag_image_wdv},
};
use kasa_core::{config::global_config::get_config_impl, tags::insert_tags_with_source_types};
use kasa_python::extractors::ExtractedTag;
use serde_json::json;
use sqlx::{query, query_as, query_scalar, sqlite::SqlitePoolOptions};

#[derive(sqlx::FromRow)]
pub struct HashAndPath {
    pub hash: String,
    pub path: String,
}

fn get_model_name(session: &Session, model_file_name: &str) -> String {
    let model_meta = session.metadata().unwrap();
    let mut model_name = model_meta.name().unwrap_or(model_file_name.to_string());

    if let Some(version) = model_meta.version() {
        model_name = format!("{}-{}", model_name, version);
    }
    model_name
}

pub async fn ai_tag_images() {
    let config = get_config_impl();

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&config.db.db_path)
        .await
        .unwrap();

    let hashes_and_paths: Vec<HashAndPath> = query_as(
        "SELECT m.hash, MIN(p.path) as path FROM Media m JOIN Path p ON p.hash = m.hash LEFT JOIN AutoTaggerInfo a ON a.hash = m.hash WHERE m.media_type = 'Image' AND a.hash IS NULL GROUP BY m.hash",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    println!("{} Hashes found", hashes_and_paths.len());

    let mut session = prepare_session(&env::var("KASA_WDV_MODEL_PATH").unwrap());
    let labels = prepare_labels(&env::var("KASA_WDV_LABEL_PATH").unwrap()).unwrap();

    let mut counter = 0;
    let hash_count = hashes_and_paths.len();

    let model_path = env::var("KASA_WDV_MODEL_PATH").unwrap().to_string();
    let model_path = PathBuf::from(model_path);

    let model_file_name = model_path.file_prefix().unwrap().to_str().unwrap();

    let model_name = get_model_name(&session, model_file_name);

    let thresholds = TaggerThresholds::default();

    // insert_tags_with_source_types is clearly not optimized for batch inserts, this will take forever no matter how
    // good your gpu is
    for hash_and_path in hashes_and_paths {
        let path: Vec<String> = query_scalar("SELECT path FROM Path WHERE hash = ?")
            .bind(&hash_and_path.hash)
            .fetch_all(&pool)
            .await
            .unwrap();

        let first_path = path.first().unwrap();

        let tags_result = tag_image_wdv(&mut session, first_path, &labels, &thresholds);
        let tags = match tags_result {
            Ok(t) => t,
            Err(e) => {
                println!(
                    "Failed to tag image {}, skipping... Error: {}",
                    first_path, e
                );
                continue;
            }
        };

        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).unwrap();

        query("INSERT INTO AutoTaggerInfo(hash, tagged_on, tagger_model, thresholds, tag_count) VALUES (?,?,?,?,?)")
            .bind(&hash_and_path.hash)
            .bind(since_epoch.as_secs() as i64)
            .bind(&model_file_name)
            .bind(serde_json::to_string(&thresholds).unwrap())
            .bind(tags.count())
            .execute(&pool).await.unwrap();

        let characters: Vec<ExtractedTag> = tags
            .character
            .iter()
            .map(|t| ExtractedTag {
                category: Some("Character".to_string()),
                tag: t.name.to_string(),
            })
            .collect();
        let general: Vec<ExtractedTag> = tags
            .general
            .iter()
            .map(|t| ExtractedTag {
                category: Some("General".to_string()),
                tag: t.name.to_string(),
            })
            .collect();

        let ratings: Vec<ExtractedTag> = vec![ExtractedTag {
            category: Some("Rating".to_string()),
            tag: tags.ratings.name,
        }];

        insert_tags_with_source_types(
            characters,
            &pool,
            Some(hash_and_path.hash.clone()),
            Some("AI Tagger".to_string()),
        )
        .await;

        insert_tags_with_source_types(
            general,
            &pool,
            Some(hash_and_path.hash.clone()),
            Some("AI Tagger".to_string()),
        )
        .await;

        insert_tags_with_source_types(
            ratings,
            &pool,
            Some(hash_and_path.hash.clone()),
            Some("AI Tagger".to_string()),
        )
        .await;
        counter += 1;
        println!("Tagged {}/{} images", counter, hash_count);
    }
}
