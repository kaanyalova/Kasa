use crate::{
    db::schema::Video,
    index::media_types::{FirstPass, MediaTypeWithData},
};
use log::error;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type, ToSchema)]
pub struct VideoAudioStreamMetadata {
    pub codec: String,
    pub codec_long_name: Option<String>,
    pub bit_rate: i64,
    pub max_rate: i64,
    pub delay: i32,
    pub rate: u32,
    pub channels: u16,
    pub format: String,
    pub frames: u64,
    pub align: u32,
    pub channel_layout: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type, ToSchema)]
pub struct VideoVideoStreamMeta {
    pub codec: String,
    pub codec_long_name: Option<String>,
    pub bit_rate: i64,
    pub max_rate: i64,
    pub delay: i32,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub has_b_frames: bool,
    pub aspect_ratio: String,
    pub color_space: String,
    pub color_range: String,
    pub color_primaries: String,
    pub color_transfer_characteristic: String,
    pub chroma_location: String,
    pub references: i32,
    pub intra_dc_precision: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type, ToSchema)]
pub struct VideoMetadata {
    pub duration: f64,
    pub bit_rate: i64,
    pub audio_meta: Option<VideoAudioStreamMetadata>,
    pub video_meta: Option<VideoVideoStreamMeta>,
}

pub fn index_video_batch(first_passes: &Vec<FirstPass>) -> Vec<MediaTypeWithData> {
    first_passes
        .into_par_iter()
        .map(|video| {
            let meta = ffmpeg::format::input(&video.path);

            let meta = match meta {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to get video metadata for {:?}: {}", video.path, e);
                    return MediaTypeWithData::Invalid(video.hash.clone());
                }
            };

            let duration = meta.duration() as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE);
            let bit_rate = meta.bit_rate() as i64;

            let mut video_meta = None;
            let mut audio_meta = None;

            if let Some(video_stream) = meta.streams().best(ffmpeg::media::Type::Video)
                && let Ok(codec_context) =
                    ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())
            {
                let codec_desc = codec_context.codec().map(|c| c.description().to_string());

                if let Ok(video) = codec_context.decoder().video() {
                    let codec_name = video.id().name().to_string();

                    video_meta = Some(VideoVideoStreamMeta {
                        codec: codec_name,
                        codec_long_name: codec_desc,
                        bit_rate: video.bit_rate() as i64,
                        max_rate: video.max_bit_rate() as i64,
                        delay: video.delay() as i32,
                        width: video.width(),
                        height: video.height(),
                        format: format!("{:?}", video.format()),
                        has_b_frames: video.has_b_frames(),
                        aspect_ratio: format!(
                            "{}:{}",
                            video.aspect_ratio().0,
                            video.aspect_ratio().1
                        ),
                        color_space: format!("{:?}", video.color_space()),
                        color_range: format!("{:?}", video.color_range()),
                        color_primaries: format!("{:?}", video.color_primaries()),
                        color_transfer_characteristic: format!(
                            "{:?}",
                            video.color_transfer_characteristic()
                        ),
                        chroma_location: format!("{:?}", video.chroma_location()),
                        references: video.references() as i32,
                        intra_dc_precision: video.intra_dc_precision(),
                    });
                }
            }

            if let Some(audio_stream) = meta.streams().best(ffmpeg::media::Type::Audio)
                && let Ok(codec_context) =
                    ffmpeg::codec::context::Context::from_parameters(audio_stream.parameters())
            {
                let codec_desc = codec_context.codec().map(|c| c.description().to_string());

                if let Ok(audio) = codec_context.decoder().audio() {
                    let codec_name = audio.id().name().to_string();

                    audio_meta = Some(VideoAudioStreamMetadata {
                        codec: codec_name,
                        codec_long_name: codec_desc,
                        bit_rate: audio.bit_rate() as i64,
                        max_rate: audio.max_bit_rate() as i64,
                        delay: audio.delay() as i32,
                        rate: audio.rate(),
                        channels: audio.channels(),
                        format: format!("{:?}", audio.format()),
                        frames: audio.frames() as u64,
                        align: audio.align() as u32,
                        channel_layout: format!("{:?}", audio.channel_layout()),
                    });
                }
            }

            let meta = VideoMetadata {
                duration,
                bit_rate,
                audio_meta,
                video_meta,
            };

            MediaTypeWithData::Video(Video {
                hash: video.hash.clone(),
                video_length: duration,
                metadata: Json(meta),
            })
        })
        .collect()
}
