use std::path::Path;

use crate::index::media_types::{FirstPass, MediaTypeWithData};
use log::{error, trace};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub fn index_video_batch(first_passes: &Vec<FirstPass>) -> Vec<MediaTypeWithData> {
    let _ = first_passes.into_par_iter().map(|video| {
        let meta = ffmpeg::format::input(&video.path);

        let meta = match meta {
            Ok(val) => val,
            Err(e) => {
                error!("Failed to get video metadata for {:?}: {}", video.path, e);
                return MediaTypeWithData::Invalid(video.hash.clone());
            }
        };

        trace!("{:?}", meta.metadata()); // whats this?

        let duration = meta.duration() as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE);
        let rate = meta.bit_rate();

        let streams = meta.streams();
        let video_stream = streams.best(ffmpeg::media::Type::Video);
        let audio_stream = streams.best(ffmpeg::media::Type::Audio);

        todo!()
    });
    todo!()
}
