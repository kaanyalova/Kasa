use log::error;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::db::schema::Image;

use super::media_types::{FirstPass, MediaTypeWithData};

/// Indexes a batch of images returning either metadata with MediaTypeWithData::Image(data)
/// or MediaTypeWithData::Invalid
pub fn index_image_batch(first_passes: &Vec<FirstPass>) -> Vec<MediaTypeWithData> {
    first_passes
        .into_par_iter()
        .map(|img| {
            //  if let resolution = imagesize::size(&img.path);
            let resolution = match imagesize::size(&img.path) {
                Ok(res) => res,
                Err(e) => {
                    error!("Failed to get image size for {}: {}", &img.path, e);
                    return MediaTypeWithData::Invalid(img.hash.clone());
                }
            };

            let image_data = Image {
                resolution_x: resolution.width.try_into().unwrap(),
                resolution_y: resolution.height.try_into().unwrap(),
                hash: img.hash.clone(),
                pixels: (resolution.width * resolution.height).try_into().unwrap(),
            };
            MediaTypeWithData::Image(image_data)
        })
        .collect()
}
