use std::fs;
/*
use kasa_core::thumbnailer::{thumbnail_image_batch, ImageToThumbnail};

use crate::ThumbnailArgs;

pub fn thumbnail(args: ThumbnailArgs) {
    let files = fs::read_dir(args.in_path).unwrap();

    let inp: Vec<ImageToThumbnail> = files
        .filter_map(|f| {
            let f = f.unwrap();
            if !f.file_type().unwrap().is_dir() {
                let imgs = ImageToThumbnail {
                    out_name: f.file_name().to_string_lossy().to_string(),
                    in_path: f.path().to_string_lossy().to_string(),
                };
                Some(imgs)
            } else {
                None
            }
        })
        .collect();
    thumbnail_image_batch(&inp, (256, 256), args.out_path, "png")
}
*/
