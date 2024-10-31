use std::collections::HashMap;

use itertools::Itertools;
use sqlx::{Pool, Sqlite};
use tokio::task::spawn_blocking;

use crate::{
    db::schema::MediaType,
    index::{
        indexer_first::index_first_batch, indexer_second::indexer_second_batch,
        media_types::FirstPass, write_to_db::write_to_db,
    },
    supported_formats::get_type,
};
use walkdir::WalkDir;

const CHUNK_SIZE: usize = 1000;

/// Inserting thousands of rows sequentially to a Sqlite database is really slow, this would be easily solved
/// by simply bulking the inserts but we have multiple tables to insert because of MediaType specific metadata
///
/// To solve this we first get some basic metadata about the files with `first_pass` then group the files into vectors
/// according to their types, in `second_pass` the grouped files are further processed with further metadata
/// (resolution,metadata for images, encoding data for videos etc.) added, `second_pass` converts the data into
/// several Vectors of structs for ease of inserting to DB. In the end we will have
///
///
/// First Pass:
/// Filters the files to only include supported types
/// Files are packed into `Vec<FirstPass>` with some basic metadata
///
/// Files are then categorized with their types to batches
///
/// Second Pass:
///
/// Files are packed into a few Vectors of structs
///
/// The main struct `DbWritableMediaDataBatch` contains:
///
/// `media_type_identifier` : It is the type of packaged data (Image, Video etc).
///
/// `Vec<MediaTypeWithData>`: Media type specific data is packed into `Vec<MediaTypeWithData>` the enum MediaTypeWithData contains
/// data to be bulk written into the db
///
/// `Vec<GenericMetadata>` : Basic metadata that is required for all media types,
///
/// All of them are bulk inserted with write_to_db_function
///
///
/// This is done in chunks so it doesn't set someones pc on fire when someone tries to index 1 million files
///
/// TODOS?: Inserting all GenericData before grouping them could be better, is this good enough? There shouldn't be
/// noticeable difference. Optimize this later!
///
/// In that case second_pass should only return Vec<MediaTypeWithData>
///
///

pub async fn index(path: &str, pool: &Pool<Sqlite>, pool_thumbs: &Pool<Sqlite>) {
    let mut walkdir = WalkDir::new(path)
        .into_iter()
        .filter_map(|p| p.ok())
        //.filter(|p| p.file_type().is_file())
        //.filter_map(|p| p.path().to_str().map(String::from))
        .peekable();

    while let Some(_) = walkdir.peek() {
        let chunk: Chunk = walkdir.by_ref().take(CHUNK_SIZE).collect();

        let first_passes = index_first_batch(chunk);

        let first_pass_groups = first_passes
            .into_iter()
            .map(|p| (get_type(&p.mime), p))
            .into_group_map();

        for (_type, group) in first_pass_groups {
            let batch = indexer_second_batch(_type, group);

            write_to_db(batch, _type, &pool, &pool_thumbs).await;
        }
    }
}

pub type Chunk = Vec<walkdir::DirEntry>;
