use std::path::{Path, PathBuf};

use image::ImageFormat;
use log::{error, warn};
use sqlx::{query, query_as, query_scalar, Pool, Sqlite};

use crate::thumbnail::thumbnailer::thumbnail_image_single;

use super::thumbnailer::{thumbnail_image_single_to_file, ThumbnailFormat};

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
    if &thumbs != "" {
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
        .with_extension(&thumbnail_format.to_string().to_lowercase());

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

            return Some(thumbnail_path);
        }
        Err(e) => {
            error!("An error occurred while processing thumbnail Error: {}", e);
            None
        }
    }
}

/// Gets the thumbnail with given hash from the db, returns the bytes
/// Creates the thumbnail and stores it into the db if the thumbnail doesn't exists
pub async fn get_thumbnail_from_db_impl(
    hash: &str,
    pool: &Pool<Sqlite>,
    pool_thumbs: &Pool<Sqlite>,
) -> Vec<u8> {
    let bytes: Option<Vec<u8>> = query_scalar("SELECT bytes FROM Thumbs WHERE hash = ?")
        .bind(hash)
        .fetch_optional(pool)
        .await
        .unwrap();

    if let Some(bytes) = bytes {
        return bytes;
    }

    // get the file path for the image to thumbnail
    let path: String = query_scalar("SELECT path FROM Path WHERE hash = ?")
        .bind(hash)
        .fetch_one(pool)
        .await
        .unwrap();

    // TODO un hardcode these
    let thumbnail = thumbnail_image_single(&path, (256, 256), &ThumbnailFormat::PNG).unwrap();

    // write the thumbnail to db
    query("INSERT INTO Thumbs(bytes) VALUES (?)")
        .bind(&thumbnail.bytes)
        .execute(pool_thumbs)
        .await
        .unwrap();

    // return the bytes
    thumbnail.bytes
}
