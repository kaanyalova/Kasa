use crate::{db::schema::MediaType, thumbnail::thumbnail_image::calculate_aspect_ratio};

pub fn get_thumbnail_size(media_type: MediaType, path: &str) -> (u32, u32) {
    let (src_x, src_y) = match media_type {
        MediaType::Image => {
            let size = imagesize::size(path).unwrap();
            (size.width as u32, size.height as u32)
        }
        MediaType::Video => todo!(),
        MediaType::Game => todo!(),
        MediaType::Unknown => todo!(),
    };
    // TODO make this configurable, make sure it matches the actual thumbnail sizes in dev
    calculate_aspect_ratio(src_x, src_y, 256, 256)
}
