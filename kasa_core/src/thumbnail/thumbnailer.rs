use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use anyhow::Result;
use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};
use image::codecs::avif::AvifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::ImageEncoder;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use thiserror::Error;

use crate::supported_formats::SUPPORTED_FORMATS;

pub struct ImageToThumbnail {
    /// Also the hash of the image
    pub out_name: String,
    pub in_path: String,
}

/// Thumbnails images
/// parallely iterates over each image in images
///
/// Each image is downscaled to `resolution` where
/// the longer side of the image is scaled to the longer size of the output resolution without changing the
/// aspect ratio
///
///
///
/// Each images path is supplied in `ImageToThumbnail.in_path`
/// Each thumbnail is outputted at ` {thumbnails_path}/{out_name}.{thumnnailer_format}`
pub fn thumbnail_image_batch(
    images: &Vec<ImageToThumbnail>,
    resolution: (u32, u32),
    thumbnails_path: PathBuf,
    thumbnailer_format: &str,
) {
    images.par_iter().for_each(|i| {
        // check if thumbnail is in the path, it should skip processing if it is in the db
        // but this might be useful in dev environments, so enable it on debug builds only
        #[cfg(debug_assertions)]
        {
            let is_thumbnail_there = thumbnails_path
                .join(format!("{}.{}", i.out_name, thumbnailer_format))
                .exists();

            if is_thumbnail_there {
                return;
            }
        }

        // check if the image format is one of the image formats supported by Image

        // This guesses mime types based on file extensions not as accurate as reading file headers,
        // but much faster
        let mime = mime_guess::from_path(&i.in_path)
            .first_or_octet_stream()
            .to_string();
        if !SUPPORTED_FORMATS.contains(&mime.as_ref()) {
            //dbg!(
            //    "file {} is unsupported by the thumbnailer, the mime was: {}",
            //    &i.in_path,
            //    mime
            //);
            return;
        }

        dbg!("thumbnailing image: {}", &i.in_path);

        let src_image = ImageReader::open(&i.in_path).unwrap().decode().unwrap();

        let (dst_x, dst_y) = calculate_aspect_ratio(
            src_image.width(),
            src_image.height(),
            resolution.0,
            resolution.1,
        );

        dbg!("thumbnail is {}x{} pixels", dst_x, dst_y);
        dbg!(
            "other vals, src_x = {}, src_y = {}, dest_max_x = {}, dest_max_y = {}",
            src_image.width(),
            src_image.height(),
            resolution.0,
            resolution.1
        );

        let src_color_type = src_image.color();

        let mut dest_img = Image::new(dst_x, dst_y, src_image.pixel_type().unwrap());

        // might be better to not create a resizer every time
        let mut resizer = Resizer::new();
        resizer.resize(&src_image, &mut dest_img, None).unwrap();

        let out_file = format!("{}.{}", i.out_name, thumbnailer_format);
        let out_path = thumbnails_path.join(out_file);

        println!("trying to output the file into {}", &out_path.display());
        let file = File::create(out_path).unwrap();
        let mut result_buf = BufWriter::new(file);

        PngEncoder::new(&mut result_buf)
            .write_image(dest_img.buffer(), dst_x, dst_y, src_color_type.into())
            .unwrap();
    })
}

#[derive(Debug, Error)]
pub enum ThumbnailerError {
    #[error("The format is unsupported by the thumbnailer, mime {0}")]
    FormatUnsupported(String),
    #[error("Something went wrong while thumbnailing image, details: {0}")]
    ImageOperationError(String),
}

/// Thumbnails a single image, returns the thumbnail size
/// Saves the image to given path
pub fn thumbnail_image_single_to_file(
    path: &str,
    out_path: &str,
    resolution: (u32, u32),
    format: &ThumbnailFormat,
) -> Result<(u32, u32)> {
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();
    if !SUPPORTED_FORMATS.contains(&mime.as_ref()) {
        //dbg!(
        //    "file {} is unsupported by the thumbnailer, the mime was: {}",
        //    &i.in_path,
        //    mime
        //);

        return Err(ThumbnailerError::FormatUnsupported(mime).into());
    }
    let src_image = ImageReader::open(path).unwrap().decode();

    let src_image = match src_image {
        Ok(img) => img,
        Err(e) => return Err(ThumbnailerError::ImageOperationError(e.to_string()).into()),
    };

    let src_color_type = src_image.color();

    let (dst_x, dst_y) = calculate_aspect_ratio(
        src_image.width(),
        src_image.height(),
        resolution.0,
        resolution.1,
    );

    let mut dest_img = Image::new(dst_x, dst_y, src_image.pixel_type().unwrap());

    let mut resizer = Resizer::new();
    resizer.resize(&src_image, &mut dest_img, None).unwrap();

    let file = File::create(out_path).unwrap();
    let mut result_buf = BufWriter::new(file);

    match format {
        ThumbnailFormat::PNG => {
            PngEncoder::new(&mut result_buf)
                .write_image(dest_img.buffer(), dst_x, dst_y, src_color_type.into())
                .unwrap();
        }
        ThumbnailFormat::JPEG => JpegEncoder::new(&mut result_buf)
            .write_image(dest_img.buffer(), dst_x, dst_y, src_color_type.into())
            .unwrap(),
        ThumbnailFormat::AVIF => AvifEncoder::new(&mut result_buf)
            .write_image(dest_img.buffer(), dst_x, dst_y, src_color_type.into())
            .unwrap(),
    }

    Ok((dst_x, dst_y))
}

pub struct Thumbnail {
    pub x: u32,
    pub y: u32,
    pub bytes: Vec<u8>,
}

/// Thumbnails a single image, returns the thumbnail size and bytes of the image
pub fn thumbnail_image_single(
    path: &str,
    resolution: (u32, u32),
    _format: &ThumbnailFormat,
) -> Result<Thumbnail> {
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();
    if !SUPPORTED_FORMATS.contains(&mime.as_ref()) {
        //dbg!(
        //    "file {} is unsupported by the thumbnailer, the mime was: {}",
        //    &i.in_path,
        //    mime
        //);

        return Err(ThumbnailerError::FormatUnsupported(mime).into());
    }
    let src_image = ImageReader::open(path).unwrap().decode();

    let src_image = match src_image {
        Ok(img) => img,
        Err(e) => return Err(ThumbnailerError::ImageOperationError(e.to_string()).into()),
    };

    let src_color_type = src_image.color();

    let (dst_x, dst_y) = calculate_aspect_ratio(
        src_image.width(),
        src_image.height(),
        resolution.0,
        resolution.1,
    );

    let mut dest_img = Image::new(dst_x, dst_y, src_image.pixel_type().unwrap());

    let mut resizer = Resizer::new();
    resizer.resize(&src_image, &mut dest_img, None).unwrap();

    let mut bytes: Vec<u8> = vec![];

    match _format {
        ThumbnailFormat::PNG => {
            PngEncoder::new(&mut bytes)
                .write_image(dest_img.buffer(), dst_x, dst_y, src_color_type.into())
                .unwrap();
        }
        ThumbnailFormat::JPEG => JpegEncoder::new(&mut bytes)
            .write_image(dest_img.buffer(), dst_x, dst_y, src_color_type.into())
            .unwrap(),
        ThumbnailFormat::AVIF => AvifEncoder::new(&mut bytes)
            .write_image(dest_img.buffer(), dst_x, dst_y, src_color_type.into())
            .unwrap(),
    }

    let thumbnail = Thumbnail {
        x: dst_x,
        y: dst_y,
        bytes,
    };

    Ok(thumbnail)
}

/// https://stackoverflow.com/a/14731922
/// Conserve aspect ratio of the original region. Useful when shrinking/enlarging
//  images to fit into a certain area.
pub fn calculate_aspect_ratio(
    src_x: u32,
    src_y: u32,
    dest_max_x: u32,
    dest_max_y: u32,
) -> (u32, u32) {
    let ratio = f64::min(
        dest_max_x as f64 / src_x as f64,
        dest_max_y as f64 / src_y as f64,
    );
    (
        (src_x as f64 * ratio as f64) as u32,
        (src_y as f64 * ratio as f64) as u32,
    )
}

#[derive(Debug, Serialize, Deserialize, EnumString, Display)]
#[serde(rename_all = "lowercase")]
pub enum ThumbnailFormat {
    PNG,
    JPEG,
    AVIF,
}
