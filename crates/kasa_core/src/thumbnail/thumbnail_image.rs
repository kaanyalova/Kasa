use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use anyhow::Result;
use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};
use image::ImageEncoder;
use image::ImageReader;
use image::codecs::png::PngEncoder;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};
use thiserror::Error;
use utoipa::ToSchema;

use crate::supported_formats::SUPPORTED_FORMATS;
use crate::thumbnail::encoding::resize_and_encode;

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
/// Each thumbnail is outputted at ` {thumbnails_path}/{out_name}.{thumbnailer_format}`
/// Unused, TODO remove
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

#[derive(Debug, FromRow, ToSchema, sqlx::Type)]
pub struct Thumbnail {
    pub x: u32,
    pub y: u32,
    pub bytes: Vec<u8>,
    pub format: ThumbnailFormat,
    pub success: bool,
}

impl Thumbnail {
    pub fn error_placeholder() -> Self {
        let error_placeholder = include_bytes!("placeholders/error_placeholder.png");
        let image = image::load_from_memory(error_placeholder).unwrap();
        let width = image.width();
        let height = image.height();

        Thumbnail {
            x: width,
            y: height,
            bytes: error_placeholder.to_vec(),
            format: ThumbnailFormat::Png,
            success: true,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.success && !self.bytes.is_empty() && self.x > 0 && self.y > 0
    }
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

    let original_image = match src_image {
        Ok(img) => img,
        Err(e) => return Err(ThumbnailerError::ImageOperationError(e.to_string()).into()),
    };

    resize_and_encode(&original_image, resolution, _format)
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
    ((src_x as f64 * ratio) as u32, (src_y as f64 * ratio) as u32)
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    EnumString,
    Display,
    ToSchema,
    Clone,
    specta::Type,
    PartialEq,
    sqlx::Type,
)]
#[serde(rename_all = "snake_case")]
pub enum ThumbnailFormat {
    Png,
    Jpeg,
    Avif,
    WebpLossless,
    WebpLossy,
}
impl ThumbnailFormat {
    pub fn to_mime(&self) -> String {
        match self {
            ThumbnailFormat::Png => "image/png".to_string(),
            ThumbnailFormat::Jpeg => "image/jpeg".to_string(),
            ThumbnailFormat::Avif => "image/avif".to_string(),
            ThumbnailFormat::WebpLossless => "image/webp".to_string(),
            ThumbnailFormat::WebpLossy => "image/webp".to_string(),
        }
    }

    pub fn from_mime(mime: &str) -> Option<Self> {
        match mime {
            "image/png" => Some(ThumbnailFormat::Png),
            "image/jpeg" => Some(ThumbnailFormat::Jpeg),
            "image/avif" => Some(ThumbnailFormat::Avif),
            "image/webp" => Some(ThumbnailFormat::WebpLossless),
            _ => None,
        }
    }
}
