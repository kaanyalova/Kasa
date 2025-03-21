use anyhow::Result;

use super::thumbnail_image::{Thumbnail, ThumbnailFormat, thumbnail_image_single};

pub fn thumbnail_group(img_paths: Vec<String>, style: GroupThumbnailStyle) -> Result<Thumbnail> {
    match style {
        GroupThumbnailStyle::FirstImage => {
            thumbnail_image_single(&img_paths[0], (256, 256), &ThumbnailFormat::PNG)
        }
    }

    /*
    let mut images = vec![];
      for img in img_paths {
        images.push(image::open(img)?);
    }
    let mut base_x = images[0].width();

    for image in &images[1..] {
        base_x += (image.width() as f32 * 0.2).ceil() as u32;
    }

    let base_y = images
        .iter()
        .max_by(|i1, i2| i1.height().cmp(&i2.height()))
        .unwrap()
        .height();
    let mut base = image::DynamicImage::new(2000, 2000, image::ColorType::Rgba8);

    let mut x_pos = 0;
    for img in images {
        imageops::overlay(&mut base, &img, x_pos, 1000);
        x_pos += 150;
    }

    Ok(base)
    */
}

#[derive(Default)]
pub enum GroupThumbnailStyle {
    #[default]
    FirstImage,
}
