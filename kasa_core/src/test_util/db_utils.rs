use sqlx::{pool, query, Pool, Sqlite};

use crate::db::schema::Media;

pub async fn insert_media_row(pool: &Pool<Sqlite>, media: &Media) {
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

pub async fn insert_hash_tag_pair_row(hash: &str, tag_name: &str, pool: &Pool<Sqlite>) {
    query("INSERT INTO HashTagPair(hash, tag_name) VALUES (?,?)")
        .bind(hash)
        .bind(tag_name)
        .execute(pool)
        .await
        .unwrap();
}
