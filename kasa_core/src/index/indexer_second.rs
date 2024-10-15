use std::{fs, os::unix::fs::MetadataExt};

use chrono::Utc;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{db::schema::MediaType, index::index_image::index_image_batch};

use super::{
    media_types::{DbWritableMediaDataBatch, FirstPass, GenericMediaData, PathData},
    thumbnail_sizes::get_thumbnail_size,
};

/// Indexes a file given its path
/// returns various info about the file to be written to DB
///
///
///
///  It checks if the hash of the file was previously written to the DB
///
/// It writes extra info about the `MediaType` of the file to a table named after that `MediaType`
pub fn indexer_second_batch(
    media_type: MediaType,
    first_passes: Vec<FirstPass>,
) -> DbWritableMediaDataBatch {
    // Any new MediaTypes can be added here along with their batch processing functions
    let media_data = match media_type {
        MediaType::Image => index_image_batch(&first_passes),
        MediaType::Video => {
            vec![] // TODO video meta
        }
        MediaType::Game => todo!(),
        MediaType::Unknown => todo!(),
    };

    let (generic_media_data, paths): (Vec<GenericMediaData>, Vec<PathData>) = first_passes
        .par_iter()
        .map(|i| {
            let thumbnail_size = get_thumbnail_size(media_type, &i.path);

            let generic_media_data = GenericMediaData {
                hash: i.hash.clone(),
                size: fs::metadata(&i.path).unwrap().size(),
                mime: i.mime.to_string(),
                thumb_path: None,
                time_added: Utc::now().timestamp_millis(),
                thumbnail_x: thumbnail_size.0 as i64,
                thumbnail_y: thumbnail_size.1 as i64,
            };

            let path_data = PathData {
                path: i.path.to_string(),
                hash: i.hash.to_string(),
            };

            (generic_media_data, path_data)
        })
        .collect();

    let out = DbWritableMediaDataBatch {
        media_type_identifier: media_type,
        media_data,
        generic_media_data,
        paths,
    };


    out
}
