use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::db::schema::Image;

use super::media_types::{FirstPass, MediaTypeWithData};

pub fn index_image_batch(first_passes: &Vec<FirstPass>) -> Vec<MediaTypeWithData> {
    first_passes
        .into_par_iter()
        .map(|img| {
            let resolution = imagesize::size(&img.path).unwrap();
            let image_data = Image {
                resolution_x: resolution.width.try_into().unwrap(),
                resolution_y: resolution.height.try_into().unwrap(),
                hash: img.hash.clone(),
            };
            MediaTypeWithData::Image(image_data)
        })
        .collect()
}
