use serde::{Deserialize, Serialize};

use crate::db::schema::Media;

/// A layout similar to google images
/// https://medium.com/@danrschlosser/building-the-image-grid-from-google-photos-6a09e193c74a
///
/// Calculates image sizes for a singe row of images, the plan is first showing random squares to user
/// then to replace them with the calculated layout
///
/// We probably want to precalculate and store the layouts for certain common "galleries" calculating this
/// for a million images every time user opens the app would be wasteful
///
/// Lets set up a max cache size like 100MB
/// We want to prioritize screens like the "All images" one where user will probably spend the most time in
///
/// We are using screen aspect ratio instead of resolution
///
/// Thumbnail sizes are downscaled using `downscale_ratio` so we can get the user to change thumbnail sizes on demand
///
/// Google images seems to use absolute positioning for all images, the blog post uses relative positioning and rows
/// ~~We also use absolute positioning because it would be much easier to cache instead of caching row positions ~~
///
/// We use relative positioning svelte-tiny-virtual-list uses item height to calculate
/// TODO check if svelte-tiny-list works with
pub fn calculate_layout(
    images: Vec<Media>,
    //downscale_mult: f64,
    width: f64,
    max_height: u64,
    gaps: u64,
) -> Vec<ImageRow> {
    /*
    let min_aspect_ratio = match width {
        0.0..=640.0 => 2.0,
        641.0..=1280.0 => 4.0,
        1281.0..=1920.0 => 5.0,
        _ => 6.0,
    };

    */

    let min_aspect_ratio = ((width / 200.0) as f64).round();

    //let max_height = match width {
    //    0.0..=640.0 => 100.0,
    //    641.0..=1920.0 => 250.0,
    //    _ => 500.0,
    //};

    //let min_aspect_ratio = width as f64 / max_height as f64;

    let mut rows: Vec<ImageRow> = vec![];

    let images_len = images.len();

    let mut current_y = 0 + gaps;

    struct TempMedia {
        hash: String,
        thumbnail_x: i64,
        thumbnail_y: i64,
    }

    // reset them
    let mut row_aspect_ratio = 0.0;
    let mut current_row_temp: Vec<TempMedia> = vec![];

    for (i, image) in images.into_iter().enumerate() {
        let image_aspect_ratio = image.thumbnail_x as f64 / image.thumbnail_y as f64;
        row_aspect_ratio += image_aspect_ratio;

        current_row_temp.push(TempMedia {
            hash: image.hash,
            thumbnail_x: image.thumbnail_x,
            thumbnail_y: image.thumbnail_y,
        });

        // Row is complete or we don't have any more images
        if image_aspect_ratio + row_aspect_ratio > min_aspect_ratio || i + 1 == images_len {
            // For the last images
            // TODO this makes the last image huge, set the image height to the max value!

            // only on last image

            // Add the values to the current row
            let row_height: f64 = width as f64 / row_aspect_ratio;

            let mut current_x = 0 + gaps; // add gaps to first one in the row

            let mut placements = vec![];

            //let width = width as u64 - (current_row_temp.len() as u64 * gaps - 1);

            for i in &current_row_temp {
                let aspect_ratio = i.thumbnail_x as f64 / i.thumbnail_y as f64;
                let width = (row_height * aspect_ratio) as u64 - gaps;
                let height = row_height as u64;

                let placement = ImagePlacement {
                    x_relative: current_x,
                    y_relative: current_y,
                    width,
                    height,
                    hash: i.hash.to_string(),
                };

                current_x += width + gaps;

                current_y += gaps; // todo remove from last image

                placements.push(placement);
            }

            let image_row = ImageRow {
                height: row_height as u64 + gaps,
                images: placements,
            };

            rows.push(image_row);
            current_y += row_height as u64;
            // reset the values
            row_aspect_ratio = 0.0;
            current_row_temp = vec![];

            // End of row end
        }
    }

    rows
}

pub struct TempRow {
    x: u64,
    y: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, specta::Type)]
pub struct ImageRow {
    height: u64,
    images: Vec<ImagePlacement>,
}

#[derive(Debug, Serialize, Deserialize, Clone, specta::Type)]
pub struct ImagePlacement {
    x_relative: u64,
    y_relative: u64,
    width: u64,
    height: u64,
    hash: String,
}
