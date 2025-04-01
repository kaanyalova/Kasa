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
            let Ok(resolution) = imagesize::size(&img.path) else {
                error!("Failed to get image size for {}", &img.path);
                return MediaTypeWithData::Invalid(img.hash.clone());
            };
            let image_data = Image {
                resolution_x: resolution.width.try_into().unwrap(),
                resolution_y: resolution.height.try_into().unwrap(),
                hash: img.hash.clone(),
            };
            MediaTypeWithData::Image(image_data)
        })
        .collect()
}
