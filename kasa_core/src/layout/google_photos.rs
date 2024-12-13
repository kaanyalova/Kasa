use serde::{Deserialize, Serialize};

use crate::{db::schema::Media, thumbnail::thumbnail_image::calculate_aspect_ratio};
use log::error;

const LAST_ROW_HEIGHT: u64 = 250;

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
///
/// TODO `max_height` is not used remove it
pub fn calculate_layout(
    images: Vec<Media>,
    //downscale_mult: f64,
    width: f64,
    _max_height: u64, // TODO remove this unused
    gaps: u64,
) -> Vec<ImageRow> {
    let mut row_index = 0;
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
                let aspect_ratio: f64 = i.thumbnail_x as f64 / i.thumbnail_y as f64;
                let mut width = (row_height * aspect_ratio) as u64 - gaps;

                // will overflow with super small images otherwise
                if width <= gaps {
                    width = gaps + 1;
                }

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

            let mut image_row = ImageRow {
                index: row_index,
                height: row_height as u64 + gaps,
                images: placements,
            };

            row_index += 1;

            // At last image, go back and resize all images to sensible sizes

            // if the last row gets resized to a width higher than the width of the screen we are fucked
            // checking `row_aspect_ratio < width / LAST_ROW_HEIGHT` should prevent that and use the regular, non-rescaled row layout instead
            // if it is the case it should have a high aspect ratio and shouldn't look out of place
            if i + 1 == images_len && row_aspect_ratio < width / LAST_ROW_HEIGHT as f64 {
                // min_aspect_ratio ~= 10 in 1080p
                let mut current_x_last = 0 + gaps;

                for image in &mut image_row.images {
                    let (x, y) =
                        // this is not efficient i know, and don't care
                        calculate_aspect_ratio(image.width as u32, image.height as u32, 99999, LAST_ROW_HEIGHT as u32);

                    image.x_relative = current_x_last;
                    current_x_last += x as u64 + gaps;

                    image.width = x as u64;
                    image.height = y as u64;
                }

                image_row.height = LAST_ROW_HEIGHT + gaps;

                // set the row height to predetermined value
                //image_row.height = LAST_ROW_HEIGHT;

                assert!(width as u64 >= current_x_last, "Width of the images are larger than the width of screen, something went wrong with last row layout. The assertion that (width = {} >= current_x_last = {}) failed", width, current_x_last);
            }

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

#[derive(Debug, Serialize, Deserialize, Clone, specta::Type)]
pub struct ImageRow {
    index: u64,
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

#[test]
fn test_random_generation() {
    use rand::Rng;

    for _test in 1..10_000 {
        let mut media = vec![];
        for _m in 1..100 {
            let rand_x = rand::thread_rng().gen_range(1..=2000);
            let rand_y = rand::thread_rng().gen_range(1..=2000);

            media.push({
                Media {
                    hash: "hash".to_string(),
                    media_type: "".to_string(),
                    thumb_path: Some("".to_string()),
                    thumbnail_x: rand_x,
                    thumbnail_y: rand_y,
                    filesize: 100,
                    mime: None,
                    time_added: 0,
                    has_file_ref: false,
                    hide: false,
                }
            });
        }

        calculate_layout(media, 1920.0, 0, 0);
    }
}
