use crate::thumbnail::thumbnail_flash::get_flash_resolution_impl;
use crate::{db::schema::MediaType, thumbnail::thumbnail_image::calculate_aspect_ratio};
use anyhow::Result;
use ffmpeg::format::input;
use ffmpeg::media::Type;

pub fn get_thumbnail_size(media_type: MediaType, path: &str) -> (u32, u32) {
    let (src_x, src_y) = match media_type {
        MediaType::Image => {
            let size = imagesize::size(path).unwrap();
            (size.width as u32, size.height as u32)
        }
        MediaType::Video => {
            get_video_resolution(path).unwrap_or((1920, 1080)) // default value if ffmpeg dies
        } // TODO ignore errors
        MediaType::Game => todo!(),
        MediaType::Unknown => todo!(),
        MediaType::Group => todo!(),
        MediaType::Flash => get_flash_resolution_impl(path).unwrap_or((256, 256)),
    };
    // TODO make this configurable, make sure it matches the actual thumbnail sizes in dev
    calculate_aspect_ratio(src_x, src_y, 256, 256)
}

fn get_video_resolution(path: &str) -> Result<(u32, u32)> {
    ffmpeg::init()?;

    let ictx = input(path)?;
    let input = ictx
        .streams()
        .best(Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;

    let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;

    let decoder = context_decoder.decoder().video()?;

    Ok((decoder.width(), decoder.height()))
}
