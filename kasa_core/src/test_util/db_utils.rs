use sqlx::{query, Pool, Sqlite};

use crate::db::schema::{Media, Path};

pub async fn _insert_media_row(pool: &Pool<Sqlite>, media: &Media) {
    // Too long SQL strings cause rustfmt to die

    let sql = "INSERT INTO Media(hash, thumb_path, media_type, filesize, mime, thumbnail_x, thumbnail_y, time_added) VALUES (?,?,?,?,?,?,?,?)";

    query(&sql)
        .bind(&media.hash)
        .bind(&media.thumb_path)
        .bind(&media.media_type)
        .bind(media.filesize)
        .bind(&media.mime)
        .bind(media.thumbnail_x)
        .bind(media.thumbnail_y)
        .bind(media.time_added)
        .execute(pool)
        .await
        .unwrap();
}

pub async fn insert_media_row(
    pool: &Pool<Sqlite>,
    hash: &str,
    thumb_path: &str,
    media_type: &str,
    filesize: i64,
    mime: &str,
    thumbnail_x: i64,
    thumbnail_y: i64,
    time_added: i64,
    has_file_ref: bool,
) {
    let media = &Media {
        hash: hash.to_string(),
        media_type: media_type.to_string(),
        thumb_path: Some(thumb_path.to_string()),
        thumbnail_x,
        thumbnail_y,
        filesize,
        mime: mime.to_string(),
        time_added,
        has_file_ref,
    };
    _insert_media_row(pool, media).await;
}

pub async fn insert_hash_tag_pair_row(hash: &str, tag_name: &str, pool: &Pool<Sqlite>) {
    query("INSERT INTO HashTagPair(hash, tag_name) VALUES (?,?)")
        .bind(hash)
        .bind(tag_name)
        .execute(pool)
        .await
        .unwrap();
}

pub async fn insert_path_row(pool: &Pool<Sqlite>, hash: &str, path: &str, imported_from: &str) {
    query("INSERT INTO Path(hash, path, imported_from) VALUES (?,?,?)")
        .bind(hash)
        .bind(path)
        .bind(imported_from)
        .execute(pool)
        .await
        .unwrap();
}
