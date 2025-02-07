use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Ok, Result};
use ruffle_render_wgpu::wgpu::hal::auxil::db;
use sqlx::{prelude::FromRow, query, query_as, query_scalar, Pool, QueryBuilder, Sqlite};
use xxhash_rust::xxh3::xxh3_64;

use crate::{
    db::schema::{media_type_to_string, Media, MediaType},
    media::{get_info_impl, MediaInfo},
    test_util::{self, db_utils::insert_media_row},
};

const MAX_BINDS: usize = 32766;

#[allow(unused)]
async fn create_group(
    media_hashes: Vec<String>,
    group_name: Option<String>,
    hide_entries: bool,
    insert_media: bool,
    db: &Pool<Sqlite>,
) -> Result<u64> {
    let sum_of_hashes = media_hashes
        .iter()
        .map(|h| h.parse::<u64>().unwrap())
        .reduce(|l, r| l.wrapping_add(r))
        .unwrap();

    let bytes = unsafe { std::mem::transmute::<u64, [u8; 8]>(sum_of_hashes) };
    let hash = xxh3_64(&bytes);

    query("INSERT INTO MediaGroup(group_hash,group_name) VALUES (?,?) ")
        .bind(group_name)
        .bind(insert_media)
        .execute(db)
        .await?;

    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT INTO MediaGroupEntry(group_id, hash)");
    let hashes = media_hashes.iter();

    query_builder.push_values(hashes.take(MAX_BINDS / 2), |b, entry| {
        //b.push_bind(&group_id).push_bind(entry);
    });

    let _query = query_builder.build();
    _query.execute(db).await?;

    if hide_entries {
        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE Media SET hide = true WHERE Media.hash IN (");

        let mut separated = query_builder.separated(", ");

        for hash in &media_hashes {
            separated.push_bind(hash);
        }

        separated.push_unseparated(") ");

        let query = query_builder.build();

        query.execute(db).await?;
    }

    if insert_media {
        // might overflow, doesn't matter though

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        #[derive(FromRow)]
        struct ThumbnailSizes {
            thumbnail_x: i64,
            thumbnail_y: i64,
        }

        // TODO: This just selects the first image as the thumbnail. Need to change this for the other generation types
        let thumbnail_sizes: ThumbnailSizes =
            query_as("SELECT thumbnail_x, thumbnail_y FROM Media WHERE hash = ?")
                .bind(&media_hashes[0])
                .fetch_one(db)
                .await?;

        query("INSERT INTO Media(hash, media_type, time_added, has_file_ref, hide, thumbnail_x, thumbnail_y) VALUES (?,?,?,?,?,?,?)")
                .bind(hash.to_string())
                .bind(media_type_to_string(&MediaType::Group))
                .bind(since_the_epoch.as_secs() as i64)
                .bind(true)
                .bind(false)
                .bind(thumbnail_sizes.thumbnail_x)
                .bind(thumbnail_sizes.thumbnail_y)
                .execute(db)
                .await?;
    }

    Ok(hash)
}

pub async fn get_group_info_impl(db: &Pool<Sqlite>, group_id: &str) -> Result<Vec<MediaInfo>> {
    let mut info = vec![];

    let hashes_of_entries: Vec<String> =
        query_scalar("SELECT * FROM MediaGroupEntry WHERE group_id = ?")
            .bind(group_id)
            .fetch_all(db)
            .await?;

    for hash in hashes_of_entries {
        let media_info = get_info_impl(&hash, db).await;

        info.push(media_info);
    }

    Ok(info)
}

#[allow(unused)]
async fn get_grouped_info(db: &Pool<Sqlite>) {}

#[allow(unused)]
async fn remove_group(group_id: &str, db: &Pool<Sqlite>) -> Result<()> {
    query("DELETE FROM MediaGroup WHERE group_id = ?")
        .bind(group_id)
        .execute(db)
        .await?;

    query("DELETE FROM MediaGroupEntry WHERE group_id = ?")
        .bind(group_id)
        .execute(db)
        .await?;

    Ok(())
}

#[sqlx::test]
async fn test_groups(pool: Pool<Sqlite>) {
    sqlx::migrate!("../migrations/db").run(&pool).await.unwrap();
    insert_media_row(
        &pool,
        "1",
        "test.jpg",
        &media_type_to_string(&MediaType::Image),
        0,
        "image/jpeg",
        0,
        0,
        0,
        true,
        false,
    )
    .await;

    insert_media_row(
        &pool,
        "2",
        "test.jpg",
        &media_type_to_string(&MediaType::Image),
        0,
        "image/jpeg",
        0,
        0,
        0,
        true,
        false,
    )
    .await;

    insert_media_row(
        &pool,
        "3",
        "test.jpg",
        &media_type_to_string(&MediaType::Image),
        0,
        "image/jpeg",
        0,
        0,
        0,
        true,
        false,
    )
    .await;

    let group_id = create_group(
        vec!["1".to_string(), "2".to_string(), "3".to_string()],
        Some("named".to_string()),
        false,
        true,
        &pool,
    );
}
