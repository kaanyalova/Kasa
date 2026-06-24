use std::{path::PathBuf, task::Poll};

use anyhow::anyhow;
use base64::prelude::*;
use log::{error, trace};
use sqlx::{Pool, Sqlite, prelude::FromRow, query, query_as, query_scalar};

use crate::{
    supported_formats,
    thumbnail::{
        thumbnail_group::thumbnail_group,
        thumbnail_image::{Thumbnail, ThumbnailerError, thumbnail_image_single},
        thumbnail_video::thumbnail_video,
    },
};

use super::{thumbnail_flash::thumbnail_flash, thumbnail_image::ThumbnailFormat};

#[derive(FromRow)]
pub struct ThumbnailData {
    pub bytes: Option<Vec<u8>>,
    success: bool,
}

impl ThumbnailData {
    pub fn is_valid(&self) -> bool {
        self.bytes.is_some() && self.success
    }
}

/// Gets the thumbnail with given hash from the db, returns base64 encoded image
/// Creates the thumbnail and stores it into the db if the thumbnail doesn't exists
pub async fn generate_or_get_thumbnail_from_db_impl(
    hash: &str,
    pool: &Pool<Sqlite>,
    pool_thumbs: &Pool<Sqlite>,
) -> Vec<u8> {
    let thumbnail_from_db = get_thumbnail_from_db_impl(hash, pool_thumbs).await;

    if let Some(encoded_thumbnail) = thumbnail_from_db
        && encoded_thumbnail.is_valid()
    {
        return encoded_thumbnail.bytes.unwrap(); // does not panic
    }

    // get the file path for the image to thumbnail
    let paths: Vec<String> = query_scalar("SELECT path FROM Path WHERE hash = ?")
        .bind(hash)
        .fetch_all(pool)
        .await
        .unwrap();

    let path = paths
        .into_iter()
        .find(|p| std::path::Path::new(p).exists())
        .unwrap_or_default();

    // TODO un hardcode these

    let mime: String = query_scalar("SELECT mime FROM Media WHERE hash = ?")
        .bind(hash)
        .fetch_one(pool)
        .await
        .unwrap();

    let _type = supported_formats::get_type(&mime);

    let thumbnail = match _type {
        crate::db::schema::MediaType::Image => {
            let path = path.clone();
            let (tx, rx) = tokio::sync::oneshot::channel();
            rayon::spawn(move || {
                let result = std::panic::catch_unwind(|| {
                    thumbnail_image_single(&path, (256, 256), &ThumbnailFormat::PNG)
                });

                let out = match result {
                    Ok(res) => res,
                    Err(_) => Err(anyhow::anyhow!("thumbnail_image_single panicked")),
                };
                let _ = tx.send(out);
            });
            rx.await.unwrap_or_else(|_| {
                Err(anyhow::anyhow!(
                    "Thumbnail generation thread failed/panicked"
                ))
            })
        }
        crate::db::schema::MediaType::Video => {
            let path = path.clone();
            let (tx, rx) = tokio::sync::oneshot::channel();
            rayon::spawn(move || {
                let result = std::panic::catch_unwind(|| {
                    thumbnail_video(&path, (256, 256), &ThumbnailFormat::PNG, 5000)
                });

                let out = match result {
                    Ok(res) => res,
                    Err(_) => Err(anyhow::anyhow!("thumbnail_video panicked")),
                };
                let _ = tx.send(out);
            });
            rx.await.unwrap_or_else(|_| {
                Err(anyhow::anyhow!(
                    "Thumbnail generation thread failed/panicked"
                ))
            })
        }
        crate::db::schema::MediaType::Game => {
            return vec![];
        }
        crate::db::schema::MediaType::Unknown => {
            error!(
                "Unknown mime type {}, you have somehow managed to index a format that wasn't on the supported formats list.",
                mime
            );
            return vec![];
        }
        crate::db::schema::MediaType::Group => {
            let hashes: Vec<String> =
                query_scalar("SELECT hash FROM MediaGroupEntry WHERE group_hash = ?")
                    .bind(hash.to_string())
                    .fetch_all(pool)
                    .await
                    .unwrap_or_default();

            let (tx, rx) = tokio::sync::oneshot::channel();
            rayon::spawn(move || {
                let result =
                    std::panic::catch_unwind(|| thumbnail_group(hashes, Default::default()));

                let out = match result {
                    Ok(res) => res,
                    Err(_) => Err(anyhow::anyhow!("thumbnail_group panicked")),
                };
                let _ = tx.send(out);
            });
            rx.await.unwrap_or_else(|_| {
                Err(anyhow::anyhow!(
                    "Thumbnail generation thread failed/panicked"
                ))
            })
        }
        crate::db::schema::MediaType::Flash => {
            thumbnail_flash(&path, (256, 256), &ThumbnailFormat::PNG).await
        }
        crate::db::schema::MediaType::Pdf => Err(anyhow!(ThumbnailerError::FormatUnsupported(
            "Pdf thumbnails not implemented yet".to_string()
        ))),
    };

    // Handle the Result<Thumbnail> outside the match statement

    let error_placeholder = include_bytes!("placeholders/error_placeholder.png");
    let thumnail_success = thumbnail.is_ok();

    let thumbnail = match thumbnail {
        Ok(thumb) => thumb,
        Err(e) => {
            error!("Failed to generate thumbnail: {}", e);

            Thumbnail {
                x: 256,
                y: 256,
                bytes: error_placeholder.to_vec(),
            }
        }
    };

    //let thumbnail = thumbnail_image_single(&path, (256, 256), &ThumbnailFormat::PNG).unwrap();

    // write the thumbnail to db
    insert_thumbnail_into_db_impl(hash, &thumbnail, pool_thumbs, thumnail_success).await;

    // return the encoded
    thumbnail.bytes
}

pub async fn insert_thumbnail_into_db_impl(
    hash: &str,
    thumbnail: &Thumbnail,
    pool: &Pool<Sqlite>,
    success: bool,
) {
    query(
        "INSERT OR REPLACE INTO Thumbs(hash, x, y, x_max, y_max, format, bytes, success) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(hash)
    .bind(thumbnail.x)
    .bind(thumbnail.y)
    .bind(256) // TODO unhardcode
    .bind(256) // TODO unhardcode
    .bind("PNG") // TODO unhardcode
    .bind(&thumbnail.bytes)
    .bind(success)
    .execute(pool)
    .await
    .unwrap();
}

pub async fn get_thumbnail_from_db_impl(
    hash: &str,
    pool_thumbs: &Pool<Sqlite>,
) -> Option<ThumbnailData> {
    query_as("SELECT bytes, success FROM Thumbs WHERE hash = ?")
        .bind(hash)
        .fetch_optional(pool_thumbs)
        .await
        .unwrap()
}
