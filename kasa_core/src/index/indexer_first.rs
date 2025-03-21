use rayon::prelude::*;

use crate::supported_formats::SUPPORTED_FORMATS;
use crate::xxhash::streaming_xxhash;

use super::indexer::Chunk;
use super::media_types::FirstPass;

/// Only indexes the hashes and types of the file
/// Inserting thousands of rows without grouping is really slow
/// so we first group the items with their types to process them in the second pass later
///
pub fn index_first_batch(chunk: Chunk) -> Vec<FirstPass> {
    chunk
        .into_par_iter()
        .filter_map(|chunk| {
            //if let Ok(f) = chunk {
            if chunk.file_type().is_file() {
                let hash = streaming_xxhash(chunk.path());
                let path = chunk.path();

                let _media = FirstPass {
                    hash: hash.to_string(),
                    path: path.to_string_lossy().to_string(),
                    mime: mime_guess::from_path(path)
                        .first_or_octet_stream()
                        .to_string(),
                };

                Some(_media)
            } else {
                // DirEntry is a path (or something like that)
                None
            }
            // } else {
            //     // DirEntry is not Ok
            //     None
            //}
        })
        .filter(|f| {
            // filter the unsupported formats out, TODO add a log here if it doesn't match
            let mime = mime_guess::from_path(&f.path)
                .first_or_octet_stream()
                .to_string();
            SUPPORTED_FORMATS.contains(&mime.as_ref())
        })
        .collect()
}
