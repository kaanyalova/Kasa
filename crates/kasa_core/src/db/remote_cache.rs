use anyhow::Result;
use sqlx::{Pool, Sqlite, query, query_scalar};

use crate::db::schema::{MediaType, media_type_to_string};

pub async fn insert_media_type_to_remote_cache(
    hash: &str,
    media_type: MediaType,
    thumbs_db: &Pool<Sqlite>,
) -> Result<()> {
    query(
        "INSERT INTO RemoteMediaCache(hash, media_type) VALUES (?, ?) ON CONFLICT(hash) DO UPDATE SET media_type = excluded.media_type",
    )
    .bind(hash)
    .bind(media_type_to_string(&media_type))
    .execute(thumbs_db)
    .await?;
    Ok(())
}

pub async fn get_media_type_from_remote_cache(
    hash: &str,
    thumbs_db: &Pool<Sqlite>,
) -> Result<Option<MediaType>> {
    let row: Option<String> =
        query_scalar("SELECT media_type FROM RemoteMediaCache WHERE hash = ?")
            .bind(hash)
            .fetch_optional(thumbs_db)
            .await?;

    if let Some(media_type_str) = row {
        let media_type = media_type_str.parse::<MediaType>()?;
        Ok(Some(media_type))
    } else {
        Ok(None)
    }
}

pub async fn insert_video_length_to_remote_cache(
    hash: &str,
    video_length: f64,
    thumbs_db: &Pool<Sqlite>,
) -> Result<()> {
    query(
        "INSERT INTO RemoteMediaCache(hash, video_length) VALUES (?, ?) ON CONFLICT(hash) DO UPDATE SET video_length = excluded.video_length"
    )
    .bind(hash)
    .bind(video_length)
    .execute(thumbs_db)
    .await?;
    Ok(())
}

pub async fn get_video_length_from_remote_cache(
    hash: &str,
    thumbs_db: &Pool<Sqlite>,
) -> Result<Option<f64>> {
    let row: Option<f64> = query_scalar("SELECT video_length FROM RemoteMediaCache WHERE hash = ?")
        .bind(hash)
        .fetch_optional(thumbs_db)
        .await?;

    // sqlite or sqlx converts REAL NULLs to 0.0???
    // not documented here https://www.sqlite.org/c3ref/column_blob.html, but float nulls are 0.0, reals might
    // be as well?
    // also https://github.com/transact-rs/sqlx/issues/3221#issuecomment-2092340064
    if row == Some(0.0) {
        return Ok(None);
    }

    Ok(row)
}

pub async fn insert_media_name_to_remote_cache(
    hash: &str,
    media_name: &str,
    thumbs_db: &Pool<Sqlite>,
) -> Result<()> {
    query(
        "INSERT INTO RemoteMediaCache(hash, filename) VALUES (?, ?) ON CONFLICT(hash) DO UPDATE SET filename = excluded.filename",
    )
    .bind(hash)
    .bind(media_name)
    .execute(thumbs_db)
    .await?;
    Ok(())
}

pub async fn get_media_name_from_remote_cache(
    hash: &str,
    thumbs_db: &Pool<Sqlite>,
) -> Result<Option<String>> {
    let row: Option<String> = query_scalar("SELECT filename FROM RemoteMediaCache WHERE hash = ?")
        .bind(hash)
        .fetch_optional(thumbs_db)
        .await?;

    Ok(row)
}
