use std::path::PathBuf;

use anyhow::anyhow;
use base64::prelude::*;
use log::{error, trace};
use sqlx::{Pool, Sqlite, prelude::FromRow, query, query_as, query_scalar};

use crate::{
    supported_formats,
    thumbnail::{
        thumbnail_group::thumbnail_group,
        thumbnail_image::{Thumbnail, thumbnail_image_single},
        thumbnail_video::thumbnail_video,
    },
};

use super::{
    thumbnail_flash::thumbnail_flash,
    thumbnail_image::{ThumbnailFormat, thumbnail_image_single_to_file},
};

/// Returns the relative path of the thumbnail inside the thumbnails directory
pub async fn get_thumbnail_from_file_impl(
    pool: &Pool<Sqlite>,
    hash: &str,
    thumbnails_path: PathBuf,
    thumbnail_format: ThumbnailFormat,
    resolution_max: (u32, u32),
) -> Option<String> {
    // Check if the thumbnail exist in the db, return that if it does
    let thumbs_path: Option<String> = query_scalar("SELECT thumb_path FROM Media WHERE hash = ?")
        .bind(hash)
        .fetch_optional(pool)
        .await
        .unwrap();

    let thumbs = thumbs_path.unwrap();
    if !thumbs.is_empty() {
        return Some(thumbs);
    }
    /*
    This does not work
        if let Some(thumb) = thumbs_path {
            println!(
                "thumb already exists for hash:{} thumbs_path:{}",
                hash, thumb
            );
            return thumb;
        }
    */
    let out_path = thumbnails_path
        .join(hash)
        .with_extension(thumbnail_format.to_string().to_lowercase());

    let path: String = query_scalar("SELECT path FROM Path WHERE hash = ?")
        .bind(hash)
        .fetch_one(pool)
        .await
        .unwrap();

    match thumbnail_image_single_to_file(
        &path,
        out_path.to_str().unwrap(),
        resolution_max,
        &thumbnail_format,
    ) {
        Ok(_size) => {
            let thumbnail_path =
                format!("{}.{}", hash, thumbnail_format.to_string().to_lowercase());

            // insert the thumbnail path into the db
            query("UPDATE Media SET thumb_path = ? WHERE hash = ?")
                .bind(&thumbnail_path)
                .bind(hash)
                .execute(pool)
                .await
                .unwrap();

            Some(thumbnail_path)
        }
        Err(e) => {
            error!("An error occurred while processing thumbnail Error: {}", e);
            None
        }
    }
}

/// Gets the thumbnail with given hash from the db, returns base64 encoded image
/// Creates the thumbnail and stores it into the db if the thumbnail doesn't exists
///
/// Stores the thumbnail in the db as raw bytes instead of base64 encoded strings because it is more
/// storage efficient
pub async fn get_thumbnail_from_db_impl(
    hash: &str,
    pool: &Pool<Sqlite>,
    pool_thumbs: &Pool<Sqlite>,
) -> String {
    #[derive(FromRow)]
    struct ThumbnailData {
        bytes: Option<Vec<u8>>,
        success: bool,
    }

    let thumbnail_from_db: Option<ThumbnailData> =
        query_as("SELECT bytes, success FROM Thumbs WHERE hash = ?")
            .bind(hash)
            .fetch_optional(pool_thumbs)
            .await
            .unwrap();

    if let Some(thumbnail) = thumbnail_from_db {
        if let Some(bytes) = thumbnail.bytes {
            if !bytes.is_empty() && thumbnail.success {
                trace!("thumbnail found in db returning that");
                return BASE64_STANDARD.encode(bytes);
            }
        }
    }

    // get the file path for the image to thumbnail
    let path: String = query_scalar("SELECT path FROM Path WHERE hash = ?")
        .bind(hash)
        .fetch_one(pool)
        .await
        .unwrap();

    // TODO un hardcode these

    let mime: String = query_scalar("SELECT mime FROM Media WHERE hash = ?")
        .bind(hash)
        .fetch_one(pool)
        .await
        .unwrap();

    let _type = supported_formats::get_type(&mime);

    let thumbnail = match _type {
        crate::db::schema::MediaType::Image => {
            thumbnail_image_single(&path, (256, 256), &ThumbnailFormat::PNG)
        }
        crate::db::schema::MediaType::Video => {
            thumbnail_video(&path, (256, 256), &ThumbnailFormat::PNG, 5000)
        }
        crate::db::schema::MediaType::Game => {
            return "".to_string(); // Return empty string for unimplemented type
        }
        crate::db::schema::MediaType::Unknown => {
            error!(
                "Unknown mime type {}, you have somehow managed to index a format that wasn't on the supported formats list.",
                mime
            );
            return "".to_string(); // Return empty string for unknown type
        }
        crate::db::schema::MediaType::Group => {
            // Handle database query errors properly
            let hashes: Vec<String> =
                query_scalar("SELECT hash FROM MediaGroupEntry WHERE group_hash = ?")
                    .bind(hash.to_string())
                    .fetch_all(pool)
                    .await
                    .unwrap(); // how to handle this ?

            thumbnail_group(hashes, Default::default())
        }
        crate::db::schema::MediaType::Flash => {
            thumbnail_flash(&path, (256, 256), &ThumbnailFormat::PNG).await
        }
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
    .bind(thumnail_success)
    .execute(pool_thumbs)
    .await
    .unwrap();

    // return the encoded
    BASE64_STANDARD.encode(thumbnail.bytes)
}
