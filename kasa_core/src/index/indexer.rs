use itertools::Itertools;
use rayon::iter::ParallelIterator;
use sqlx::{Pool, Sqlite};
use walkdir::WalkDir;

use crate::{
    db::schema::MediaType,
    index::{
        indexer_first::index_first_batch, indexer_second::indexer_second_batch,
        media_types::FirstPass, write_to_db::write_to_db,
    },
    supported_formats::get_type,
};

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
pub type Chunk = Vec<Result<walkdir::DirEntry, walkdir::Error>>;

pub async fn index(path: String, pool: Pool<Sqlite>, pool_thumbs: Pool<Sqlite>) {
    let walkdir = WalkDir::new(path);
    for file_chunk in &walkdir.into_iter().chunks(CHUNK_SIZE) {
        let chunk: Chunk = file_chunk.collect();

        let first_pass = index_first_batch(chunk);

        // We might check for hash duplicates here to skip the second pass if the file is same
        // but the user is unlikely to have a lot of duplicates in same directory, it is not worth
        // to check every file just to find a few duplicates

        // Here we construct "groups" of files to be sent to second_pass
        type MediaGroup = Vec<(MediaType, Vec<FirstPass>)>;

        // Group the using mime types to be passed into the second pass
        let first_pass_grouped: MediaGroup = first_pass
            .into_iter()
            .chunk_by(|i: &FirstPass| get_type(i.mime.as_str()))
            .into_iter()
            .map(|(_type, group_iter)| (_type, group_iter.collect()))
            .collect();

        for (_type, group) in first_pass_grouped {
            // Process the batched `FirstPass`es into second_batches
            let batch = indexer_second_batch(_type, group);
            // write them to the db
            write_to_db(batch, _type, &pool, &pool_thumbs).await;
        }
    }
}
