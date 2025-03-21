use std::env;

use kasa_core::{
    ai_tagger::{prepare_labels, prepare_session, tag_image_wdv},
    config::global_config::get_config_impl,
    tags::insert_tags_with_source_types,
};
use kasa_python::ExtractedTag;
use sqlx::{query_scalar, sqlite::SqlitePoolOptions};

pub async fn ai_tag_images() {
    let config = get_config_impl();

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&config.db.db_path)
        .await
        .unwrap();

    let hashes: Vec<String> = query_scalar("SELECT hash FROM Media WHERE media_type = 'Image'")
        .fetch_all(&pool)
        .await
        .unwrap();

    println!("{} Hashes found", hashes.len());

    let session = prepare_session(&env::var("KASA_WDV_MODEL_PATH").unwrap());
    let labels = prepare_labels(&env::var("KASA_WDV_LABEL_PATH").unwrap());

    let mut counter = 0;
    let hash_count = hashes.len();
    for hash in hashes {
        let path: Vec<String> = query_scalar("SELECT path FROM Path WHERE hash = ?")
            .bind(&hash)
            .fetch_all(&pool)
            .await
            .unwrap();

        let first_path = path.first().unwrap();

        let tags = tag_image_wdv(&session, first_path, &labels, 0.85, 0.35);

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
            Some(hash.clone()),
            Some("AI Tagger".to_string()),
        )
        .await;

        insert_tags_with_source_types(
            general,
            &pool,
            Some(hash.clone()),
            Some("AI Tagger".to_string()),
        )
        .await;

        insert_tags_with_source_types(
            ratings,
            &pool,
            Some(hash.clone()),
            Some("AI Tagger".to_string()),
        )
        .await;
        counter += 1;
        println!("Tagged {}/{} images", counter, hash_count);
    }
}
