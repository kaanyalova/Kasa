use anyhow::anyhow;
use lazy_static::lazy_static;
use log::error;
use sqlx::{Pool, Sqlite, prelude::FromRow, query, query_as, query_scalar};

use crate::{
    config::global_config::get_config_impl,
    supported_formats,
    thumbnail::{
        thumbnail_group::thumbnail_group,
        thumbnail_image::{Thumbnail, ThumbnailerError, thumbnail_image_single},
        thumbnail_video::thumbnail_video,
    },
};

lazy_static! {
    static ref DEFAULT_THUMBNAIL_FORMAT: ThumbnailFormat =
        get_config_impl().thumbs.thumbnail_format;
}

pub const WEBP_LOSSY_QUALITY: f32 = 70.0;

use super::{thumbnail_flash::thumbnail_flash, thumbnail_image::ThumbnailFormat};

/// either generates or gets a thumbnail from the db, generated thumbnails get inserted into the db
pub async fn generate_or_get_thumbnail_from_db_impl(
    hash: &str,
    pool: &Pool<Sqlite>,
    pool_thumbs: &Pool<Sqlite>,
) -> Thumbnail {
    let thumbnail_from_db = get_thumbnail_from_db_impl(hash, pool_thumbs).await;

    if let Some(encoded_thumbnail) = thumbnail_from_db
        && encoded_thumbnail.is_valid()
    {
        return encoded_thumbnail; // does not panic
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
                    thumbnail_image_single(&path, (256, 256), &DEFAULT_THUMBNAIL_FORMAT)
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
                    thumbnail_video(&path, (256, 256), &DEFAULT_THUMBNAIL_FORMAT, 5000)
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
        crate::db::schema::MediaType::Game => return Thumbnail::error_placeholder(),
        crate::db::schema::MediaType::Unknown => {
            error!(
                "Unknown mime type {}, you have somehow managed to index a format that wasn't on the supported formats list.",
                mime
            );
            return Thumbnail::error_placeholder();
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
            thumbnail_flash(&path, (256, 256), &DEFAULT_THUMBNAIL_FORMAT).await
        }
        crate::db::schema::MediaType::Pdf => Err(anyhow!(ThumbnailerError::FormatUnsupported(
            "Pdf thumbnails not implemented yet".to_string()
        ))),
    };

    // Handle the Result<Thumbnail> outside the match statement

    let thumbnail = match thumbnail {
        Ok(thumb) => thumb,
        Err(e) => {
            error!("Failed to generate thumbnail: {}", e);

            let mut thumbnail = Thumbnail::error_placeholder();
            thumbnail.success = false;

            thumbnail
        }
    };

    //let thumbnail = thumbnail_image_single(&path, (256, 256), &ThumbnailFormat::PNG).unwrap();

    // write the thumbnail to db
    insert_thumbnail_into_db_impl(hash, &thumbnail, pool_thumbs).await;

    // return the encoded
    thumbnail
}

pub async fn insert_thumbnail_into_db_impl(hash: &str, thumbnail: &Thumbnail, pool: &Pool<Sqlite>) {
    query(
        "INSERT OR REPLACE INTO Thumbs(hash, x, y, x_max, y_max, format, bytes, success) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(hash)
    .bind(thumbnail.x)
    .bind(thumbnail.y)
    .bind(256) // TODO unhardcode
    .bind(256) // TODO unhardcode
    .bind(thumbnail.format.to_mime())
    .bind(&thumbnail.bytes)
    .bind(thumbnail.success)
    .execute(pool)
    .await
    .unwrap();
}

pub async fn get_thumbnail_from_db_impl(
    hash: &str,
    pool_thumbs: &Pool<Sqlite>,
) -> Option<Thumbnail> {
    #[derive(Debug, FromRow)]
    struct ThumbnailRow {
        x: u32,
        y: u32,
        bytes: Vec<u8>,
        format: String,
        success: bool,
    }

    let row: Option<ThumbnailRow> =
        query_as("SELECT x, y, bytes, format, success FROM Thumbs WHERE hash = ?")
            .bind(hash)
            .fetch_optional(pool_thumbs)
            .await
            .unwrap();

    row.map(|r| Thumbnail {
        x: r.x,
        y: r.y,
        bytes: r.bytes,
        format: ThumbnailFormat::from_mime(&r.format).unwrap(),
        success: r.success,
    })
}
