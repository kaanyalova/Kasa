use std::path::PathBuf;

use kasa_core::thumbnail::{
    get_thumbnail::{get_thumbnail_from_db_impl, get_thumbnail_from_file_impl},
    thumbnailer::ThumbnailFormat,
};
use log::trace;
use tauri::{AppHandle, Manager};

use crate::db::DbStore;

#[tauri::command]
#[specta::specta]
pub async fn get_thumbnail(hash: String, handle: AppHandle) -> Result<Option<String>, ()> {
    //let dev_thumbs_path = fs::canonicalize("../__dev_thumbs").unwrap();
    let dev_thumbs_path = std::env::var("KASA_THUMBS_PATH").expect("KASA_THUMBS_PATH env variable was not set, it is required to provide the path for thumbnails for now!");
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        let thumbnail = get_thumbnail_from_file_impl(
            pool,
            &hash,
            PathBuf::from(dev_thumbs_path),
            ThumbnailFormat::PNG,
            (256, 256),
        )
        .await;

        return Ok(thumbnail);
    } else {
        Ok(None)
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_thumbnail_from_db(hash: String, handle: AppHandle) -> Option<String> {
    trace!("getting thumbnail for hash:{}", hash);
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;
    let connection_guard_thumbs = connection_state.thumbs_db.lock().await;

    if let (Some(pool), Some(pool_thumbs)) =
        (connection_guard.as_ref(), connection_guard_thumbs.as_ref())
    {
        return Some(get_thumbnail_from_db_impl(&hash, pool, pool_thumbs).await);
    }

    trace!("something went wrong when thumbnailing");
    None
}

/*
#[tauri::command]
pub async fn get_thumbnails(
    page: i64,
    count: i64,
    handle: AppHandle,
) -> Result<Option<Vec<TestMedia>>, ()> {
    let top_path = fs::canonicalize("../thumbs").unwrap();

    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;
    if let Some(pool) = connection_guard.as_ref() {
        let media = query_all_test_impl(count, page, pool).await;

        let to_thumbnail = media
            .clone()
            .into_iter()
            .filter_map(|media: TestMedia| {
                // DON'T thumbnail images with thumbnails
                if let Some(path) = &media.thumbs_path {
                    // DO thumbnail the image if the referenced thumbnail doesn't exist
                    if !Path::new(&top_path.join(path)).exists() {
                        return Some(media);
                    };

                    return None;
                }
                return Some(media);
            })
            .map(|media| ImageToThumbnail {
                out_name: media.hash,
                in_path: media.path,
            })
            .collect();

        thumbnail_image_batch(
            &to_thumbnail,
            THUMBNAIL_SIZE,
            PathBuf::from(top_path),
            THUMBNAIL_FORMAT,
        );

        // add the to_thumbnail image thumbnail names to db

        let _pool = pool.clone();
        //tokio::spawn(async move {
        for thumbnail in to_thumbnail {
            let name = format!("{}.{}", thumbnail.out_name, THUMBNAIL_FORMAT);
            query("UPDATE TestMedia SET thumbs_path = ? WHERE hash = ?")
                .bind(name)
                .bind(thumbnail.out_name)
                .execute(&_pool)
                .await
                .unwrap();
        }
        //    })
        //.await
        //.unwrap();

        return Ok(Some(media));
    } else {
        Ok(None)
    }
}
 */
