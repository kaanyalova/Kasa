use anyhow::Result;
use exif::Rational;
use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, PixelType, Resizer};
use ffmpeg::ffi::av_rescale;
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use ffmpeg::Rescale;
use image::codecs::avif::AvifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ImageBuffer, ImageEncoder, Rgb, RgbImage};
use log::debug;
use serde::de;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use super::thumbnail_image::{
    calculate_aspect_ratio, Thumbnail, ThumbnailFormat, ThumbnailerError,
};

// Returns the frame and (width, height)
fn extract_frame(input_path: &str, timestamp: i64) -> Result<(Video, (u32, u32))> {
    ffmpeg::init().unwrap();

    let mut ictx = input(&input_path)?;
    let input = ictx
        .streams()
        .best(Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;
    let video_stream_index = input.index();

    let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;

    let mut decoder = context_decoder.decoder().video()?;

    let mut scaler = Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        decoder.width().next_multiple_of(32),  // This is broken
        decoder.height().next_multiple_of(32), // https://github.com/zmwangx/rust-ffmpeg/issues/198
        Flags::BILINEAR,
    )?;

    let time_base = decoder.time_base();

    // this doesn't work on some videos, it just selects the default frame 0, decoder.time_base() also shows 0/1 on some videos
    // what is going on, is the conversion from c struct to rust broken?
    let ts = timestamp * 1000; // what??? https://github.com/pop-os/cosmic-player/blob/52b9439ca4ff4d2daeefc18ea5ba90cc8c36886c/src/player.rs#L608

    // TODO, special handling for really short videos
    ictx.seek(ts, ..)?;

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;
            let mut decoded = Video::empty();

            if decoder.receive_frame(&mut decoded).is_ok() {
                let mut rgb_frame = Video::empty();
                scaler.run(&decoded, &mut rgb_frame)?;
                //save_frame(&rgb_frame, "output_path").unwrap();
                return Ok((
                    rgb_frame,
                    (
                        decoder.width().next_multiple_of(32),
                        decoder.height().next_multiple_of(32),
                    ),
                ));
            }
        }
    }

    return Err(ThumbnailerError::ImageOperationError(
        "FFmpeg did not find any streams".to_string(),
    )
    .into());
}

fn save_frame(frame: &Video, output_path: &str) -> Result<()> {
    let mut file = File::create(output_path)?;
    file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    file.write_all(frame.data(0))?;
    Ok(())
}

fn get_buffer(frame: &Video) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>> {
    //std::fs::write("data.bin", frame.data(0).to_vec()).unwrap();
    let image = RgbImage::from_raw(frame.width(), frame.height(), frame.data(0).to_vec());
    match image {
        Some(img) => return Ok(img),
        None => Err(ThumbnailerError::ImageOperationError(
            "RgbImage::from_raw has the wrong resolution".to_string(),
        )
        .into()),
    }
}

/// Timestamp in milliseconds
pub fn thumbnail_video(
    path: &str,
    resolution: (u32, u32),
    format: &ThumbnailFormat,
    timestamp: u64,
) -> Result<Thumbnail> {
    let (frame, (width, height)) = extract_frame(path, timestamp as i64)?;
    let mut buffer = get_buffer(&frame)?;

    let (target_width, target_height) =
        calculate_aspect_ratio(width, height, resolution.0, resolution.1);

    let input_image = DynamicImage::ImageRgb8(buffer);
    let src_color_type = input_image.color();

    let mut dest_image = Image::new(
        target_width,
        target_height,
        input_image
            .pixel_type()
            .ok_or("No pixel type in the source image found")
            .map_err(|e| ThumbnailerError::ImageOperationError(e.to_string()))?,
    );

    let mut resizer = Resizer::new();
    resizer.resize(&input_image, &mut dest_image, None)?;

    let mut bytes = vec![];

    match format {
        ThumbnailFormat::PNG => {
            PngEncoder::new(&mut bytes)
                .write_image(
                    dest_image.buffer(),
                    target_width,
                    target_height,
                    src_color_type.into(),
                )
                .unwrap();
        }
        ThumbnailFormat::JPEG => JpegEncoder::new(&mut bytes)
            .write_image(
                dest_image.buffer(),
                target_width,
                target_height,
                src_color_type.into(),
            )
            .unwrap(),
        ThumbnailFormat::AVIF => AvifEncoder::new(&mut bytes)
            .write_image(
                dest_image.buffer(),
                target_width,
                target_height,
                src_color_type.into(),
            )
            .unwrap(),
    }

    let thumbnail = Thumbnail {
        x: target_width,
        y: target_height,
        bytes,
    };

    Ok(thumbnail)
}

#[test]
fn test_frames() {
    let thumb = thumbnail_video("/home/kaan/Videolar/ffmpeg_playground/p4/Persona.4.the.Golden.Animation.S01E01.1080p.SHAHID.WEB-DL.JPN.AAC2.0.H.264.MSubs-ToonsHub.mkv", (256,256), &ThumbnailFormat::PNG, 100_000).unwrap();
    std::fs::write("out.png", thumb.bytes).unwrap();
}
