use std::env;

use kasa_ai::{
    prepare_session,
    wdv_tagger::{prepare_labels, tag_image_wdv},
};
use kasa_core::{config::global_config::get_config_impl, tags::insert_tags_with_source_types};
use kasa_python::ExtractedTag;
use sqlx::{query_as, query_scalar, sqlite::SqlitePoolOptions};

#[derive(sqlx::FromRow)]
struct HashAndPath {
    hash: String,
    path: String,
}

pub async fn ai_tag_images() {
    let config = get_config_impl();

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&config.db.db_path)
        .await
        .unwrap();

    let hashes_and_paths: Vec<HashAndPath> = query_as(
        "SELECT m.hash ,path FROM Media m JOIN Path p ON p.hash = m.hash WHERE media_type = 'Image' GROUP BY p.hash",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    println!("{} Hashes found", hashes_and_paths.len());

    let mut session = prepare_session(&env::var("KASA_WDV_MODEL_PATH").unwrap());
    let labels = prepare_labels(&env::var("KASA_WDV_LABEL_PATH").unwrap());

    let mut counter = 0;
    let hash_count = hashes_and_paths.len();

    // insert_tags_with_source_types is clearly not optimized for batch inserts, this will take forever no matter how
    // good your gpu is
    for hash_and_path in hashes_and_paths {
        let path: Vec<String> = query_scalar("SELECT path FROM Path WHERE hash = ?")
            .bind(&hash_and_path.hash)
            .fetch_all(&pool)
            .await
            .unwrap();

        let first_path = path.first().unwrap();

        let tags = tag_image_wdv(&mut session, first_path, &labels, 0.85, 0.35);

        let characters: Vec<ExtractedTag> = tags
            .character
            .iter()
            .map(|t| ExtractedTag {
                _type: "Character".to_string(),
                name: t.name.to_string(),
            })
            .collect();
        let general: Vec<ExtractedTag> = tags
            .general
            .iter()
            .map(|t| ExtractedTag {
                _type: "General".to_string(),
                name: t.name.to_string(),
            })
            .collect();

        let ratings: Vec<ExtractedTag> = vec![ExtractedTag {
            _type: "Rating".to_string(),
            name: tags.ratings.name,
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
