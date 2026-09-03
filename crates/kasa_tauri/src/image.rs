use crate::db::{DatabaseState, DbStore};
use base64::prelude::*;
use kasa_core::thumbnail::{
    thumbnail_image::{Thumbnail, ThumbnailFormat},
    thumbnailer::{
        generate_or_get_thumbnail_from_db_impl, get_thumbnail_from_db_impl,
        insert_thumbnail_into_db_impl,
    },
};
use log::trace;
use tauri::{AppHandle, Manager};

#[tauri::command(async)]
#[specta::specta]
pub async fn get_thumbnail_from_db(hash: String, handle: AppHandle) -> Option<String> {
    trace!("getting thumbnail for hash:{}", hash);
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(db_store) => {
            if let (Some(pool), Some(pool_thumbs)) =
                (db_store.db.as_ref(), db_store.thumbs_db.as_ref())
            {
                let image = generate_or_get_thumbnail_from_db_impl(&hash, pool, pool_thumbs).await;

                return Some(format!(
                    r#"data:{};base64,{}"#,
                    image.format.to_mime(),
                    BASE64_STANDARD.encode(image.bytes)
                ));
            }
        }
        DbStore::Remote(remote_store) => {
            if let Some(pool_thumbs) = remote_store.thumbs_db.as_ref() {
                let local_thumbnail = get_thumbnail_from_db_impl(&hash, pool_thumbs).await;

                if let Some(thumbnail) = local_thumbnail
                    && thumbnail.is_valid()
                {
                    return Some(format!(
                        r#"data:{};base64,{}"#,
                        thumbnail.format.to_mime(),
                        BASE64_STANDARD.encode(thumbnail.bytes)
                    ));
                } else {
                    // local thumbnail cache does't have the thumbnail get it from the server and cache it
                    let bytes = remote_store.client.get_thumbnail(&hash).await.unwrap();

                    // we need to figure out the size of the image and the mime type now
                    let image = image::load_from_memory(&bytes).unwrap();
                    let width = image.width();
                    let height = image.height();

                    let format_ = image::guess_format(&bytes).unwrap();

                    let format_ = match format_ {
                        image::ImageFormat::Png => ThumbnailFormat::Png,
                        image::ImageFormat::Jpeg => ThumbnailFormat::Jpeg,
                        image::ImageFormat::Avif => ThumbnailFormat::Avif,
                        image::ImageFormat::WebP => ThumbnailFormat::WebpLossy,
                        _ => ThumbnailFormat::Png,
                    };

                    insert_thumbnail_into_db_impl(
                        &hash,
                        &Thumbnail {
                            bytes: bytes.clone(),
                            x: width,
                            y: height,
                            format: format_.clone(),
                            success: true, // is it always successful? TODO implement regenerating of broken thumbnails
                        },
                        pool_thumbs,
                    )
                    .await;

                    return Some(format!(
                        r#"data:{};base64,{}"#,
                        format_.to_mime(),
                        BASE64_STANDARD.encode(bytes)
                    ));
                }
            }
        }
        _ => panic!("db not initialized"),
    }

    trace!("something went wrong when thumbnailing");
    None
}
