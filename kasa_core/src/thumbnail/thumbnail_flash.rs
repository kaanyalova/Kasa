use anyhow::{anyhow, Result};
use fast_image_resize::{images::Image, IntoImageView, Resizer};
use image::{
    codecs::{avif::AvifEncoder, jpeg::JpegEncoder, png::PngEncoder},
    DynamicImage, ImageEncoder, RgbaImage,
};
use std::path::Path;

use super::thumbnail_image::{
    calculate_aspect_ratio, Thumbnail, ThumbnailFormat, ThumbnailerError,
};

#[derive(Debug, Copy, Clone)]
struct SizeOpt {
    /// The amount to scale the page size with
    scale: f64,

    /// Optionally override the output width
    width: Option<u32>,

    /// Optionally override the output height
    height: Option<u32>,
}

impl Default for SizeOpt {
    fn default() -> Self {
        Self {
            scale: 1.0,
            width: None,
            height: None,
        }
    }
}

/// Based on https://github.com/ruffle-rs/ruffle/blob/master/exporter/src/main.rs
// Returns a vector of images and the width and height of the images
#[cfg(feature = "swf_thumbnailer")]
async fn take_screenshot(
    //descriptors: Arc<Descriptors>,
    swf_path: &Path,
    frames: u32,
    skipframes: u32,
    size: SizeOpt,
    skip_unsupported: bool,
) -> Result<(Vec<RgbaImage>, (i32, i32))> {
    use ruffle_core::limits::ExecutionLimit;
    use ruffle_core::tag_utils::SwfMovie;
    use ruffle_core::PlayerBuilder;
    use ruffle_render_wgpu::backend::{request_adapter_and_device, WgpuRenderBackend};
    use ruffle_render_wgpu::clap::PowerPreference;
    use ruffle_render_wgpu::descriptors::Descriptors;
    use ruffle_render_wgpu::target::TextureTarget;
    use ruffle_render_wgpu::wgpu;
    use std::panic::catch_unwind;
    use std::sync::Arc;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        ..Default::default()
    });

    let (adapter, device, queue) = request_adapter_and_device(
        Default::default(),
        &instance,
        None,
        PowerPreference::Low.into(),
        None,
    )
    .await
    .map_err(|e| anyhow!(e.to_string()))?;

    let descriptors = Arc::new(Descriptors::new(instance, adapter, device, queue));

    let movie = SwfMovie::from_path(swf_path, None).map_err(|e| anyhow!(e.to_string()))?;

    if movie.is_action_script_3() && skip_unsupported {
        return Err(anyhow!("Skipping unsupported movie"));
    }

    let width = size
        .width
        .map(f64::from)
        .unwrap_or_else(|| movie.width().to_pixels());
    let width = (width * size.scale).round() as u32;

    let height = size
        .height
        .map(f64::from)
        .unwrap_or_else(|| movie.height().to_pixels());
    let height = (height * size.scale).round() as u32;

    let target = TextureTarget::new(&descriptors.device, (width, height))
        .map_err(|e| anyhow!(e.to_string()))?;
    let player = PlayerBuilder::new()
        .with_renderer(
            WgpuRenderBackend::new(descriptors, target).map_err(|e| anyhow!(e.to_string()))?,
        )
        .with_movie(movie)
        .with_viewport_dimensions(width, height, size.scale)
        .build();

    let mut result = Vec::new();
    let totalframes = frames + skipframes;

    for i in 0..totalframes {
        player.lock().unwrap().preload(&mut ExecutionLimit::none());

        player.lock().unwrap().run_frame();
        if i >= skipframes {
            let image = || {
                player.lock().unwrap().render();
                let mut player = player.lock().unwrap();
                let renderer = player
                    .renderer_mut()
                    .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
                    .unwrap();
                renderer.capture_frame()
            };
            match catch_unwind(image) {
                Ok(Some(image)) => result.push(image),
                Ok(None) => return Err(anyhow!("Unable to capture frame {} of {:?}", i, swf_path)),
                Err(e) => {
                    return Err(anyhow!(
                        "Unable to capture frame {} of {:?}: {:?}",
                        i,
                        swf_path,
                        e
                    ))
                }
            }
        }
    }
    Ok((result, (width as i32, height as i32)))
}

#[cfg(not(feature = "swf_thumbnailer"))]
async fn take_screenshot(
    //descriptors: Arc<Descriptors>,
    swf_path: &Path,
    frames: u32,
    skipframes: u32,
    size: SizeOpt,
    skip_unsupported: bool,
) -> Result<Vec<RgbaImage>> {
    let bytes = include_bytes!("swf_placeholder.png");
    let img = image::load_from_memory(bytes)?;
    Ok(vec![img.to_rgba8()])
}

/// Timestamp in milliseconds
pub async fn thumbnail_flash(
    path: &str,
    resolution: (u32, u32),
    format: &ThumbnailFormat,
) -> Result<Thumbnail> {
    let (buffer, (width, height)): (Vec<RgbaImage>, (i32, i32)) =
        take_screenshot(Path::new(&path), 1, 0, Default::default(), true).await?;

    let (target_width, target_height) =
        calculate_aspect_ratio(width as u32, height as u32, resolution.0, resolution.1);

    let input_image = DynamicImage::ImageRgba8(buffer[0].clone()); // why the clone??
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

#[cfg(feature = "swf_thumbnailer")]
pub fn get_flash_resolution_impl(path: &str) -> Result<(u32, u32)> {
    use ruffle_core::tag_utils::SwfMovie;

    let movie = SwfMovie::from_path(path, None).map_err(|e| anyhow!(e.to_string()))?;

    return Ok((
        movie.width().to_pixels() as u32,
        movie.height().to_pixels() as u32,
    ));
}

#[cfg(not(feature = "swf_thumbnailer"))]
pub fn get_flash_resolution(path: &str) -> Result<(u32, u32)> {
    return Ok((256, 256));
}
