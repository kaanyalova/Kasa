use anyhow::Result;
use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};
use image::codecs::avif::AvifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{
    ColorType, DynamicImage, ExtendedColorType, GrayAlphaImage, GrayImage, ImageEncoder, RgbImage,
    RgbaImage,
};

use super::thumbnail_image::{Thumbnail, ThumbnailFormat, calculate_aspect_ratio};
use super::thumbnailer::WEBP_LOSSY_QUALITY;

pub fn encode_from_resized_buffer(
    buffer: &[u8],
    width: u32,
    height: u32,
    src_color_type: ColorType,
    format: &ThumbnailFormat,
) -> Result<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();

    match format {
        ThumbnailFormat::Png => {
            PngEncoder::new(&mut bytes).write_image(buffer, width, height, src_color_type.into())?
        }
        ThumbnailFormat::Avif => AvifEncoder::new(&mut bytes).write_image(
            buffer,
            width,
            height,
            src_color_type.into(),
        )?,
        ThumbnailFormat::WebpLossless => WebPEncoder::new_lossless(&mut bytes).write_image(
            buffer,
            width,
            height,
            src_color_type.into(),
        )?,
        ThumbnailFormat::Jpeg => {
            let alpha_removed =
                dynamic_image_from_raw_bytes(buffer, width, height, src_color_type)?.to_rgb8();
            JpegEncoder::new(&mut bytes).write_image(
                &alpha_removed,
                width,
                height,
                ExtendedColorType::Rgb8,
            )?;
        }
        ThumbnailFormat::WebpLossy => {
            let webp_bytes = match src_color_type {
                ColorType::Rgb8 => webp::Encoder::from_rgb(buffer, width, height)
                    .encode(WEBP_LOSSY_QUALITY)
                    .to_vec(),
                ColorType::Rgba8 => webp::Encoder::from_rgba(buffer, width, height)
                    .encode(WEBP_LOSSY_QUALITY)
                    .to_vec(),
                _ => {
                    // as a fallback try converting it info an image::Image then see if that works
                    let rgba = dynamic_image_from_raw_bytes(buffer, width, height, src_color_type)?
                        .to_rgba8();
                    webp::Encoder::from_rgba(&rgba, width, height)
                        .encode(WEBP_LOSSY_QUALITY)
                        .to_vec()
                }
            };
            bytes = webp_bytes;
        }
    }

    Ok(bytes)
}

pub fn resize_and_encode(
    image: &DynamicImage,
    resolution: (u32, u32),
    format: &ThumbnailFormat,
) -> Result<Thumbnail> {
    let src_color_type = image.color();
    let (target_width, target_height) =
        calculate_aspect_ratio(image.width(), image.height(), resolution.0, resolution.1);

    let mut dest_image = Image::new(
        target_width,
        target_height,
        image
            .pixel_type()
            .ok_or(anyhow::anyhow!("no pixel type in source image"))?,
    );

    let mut resizer = Resizer::new();
    resizer.resize(image, &mut dest_image, None)?;

    let bytes = encode_from_resized_buffer(
        dest_image.buffer(),
        target_width,
        target_height,
        src_color_type,
        format,
    )?;

    Ok(Thumbnail {
        x: target_width,
        y: target_height,
        bytes,
        format: format.clone(),
        success: true,
    })
}

fn dynamic_image_from_raw_bytes(
    buffer: &[u8],
    width: u32,
    height: u32,
    color_type: ColorType,
) -> Result<DynamicImage> {
    match color_type {
        ColorType::Rgb8 => Ok(DynamicImage::ImageRgb8(
            RgbImage::from_raw(width, height, buffer.to_vec())
                .ok_or(anyhow::anyhow!("bad buffer"))?,
        )),
        ColorType::Rgba8 => Ok(DynamicImage::ImageRgba8(
            RgbaImage::from_raw(width, height, buffer.to_vec())
                .ok_or(anyhow::anyhow!("bad buffer"))?,
        )),
        ColorType::L8 => Ok(DynamicImage::ImageLuma8(
            GrayImage::from_raw(width, height, buffer.to_vec())
                .ok_or(anyhow::anyhow!("bad buffer"))?,
        )),
        ColorType::La8 => Ok(DynamicImage::ImageLumaA8(
            GrayAlphaImage::from_raw(width, height, buffer.to_vec())
                .ok_or(anyhow::anyhow!("bad buffer"))?,
        )),
        other => Err(anyhow::anyhow!("unsupported color type: {:?}", other)),
    }
}
