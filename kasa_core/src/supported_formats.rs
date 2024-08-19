use crate::db::schema::{Media, MediaType};

pub const SUPPORTED_FORMATS: [&str; 14] = [
    "image/avif", //some files may cause problesms see fox.profile0.8bpc.yuv420.odd-width.odd-height.avif
    "image/bmp",
    "image/vnd.ms-dds",
    "image/gif",
    "image/vnd.microsoft.icon",
    "image/jpeg",
    "image/x-exr",
    "image/png",
    "image/x-portable-bitmap",
    "image/x-portable-anymap",
    "image/x-targa",
    "image/x-tga",
    "image/tiff",
    "image/webp",
    // TODOS
];

pub const SUPPORTED_FORMATS_IMAGE: [&str; 14] = [
    "image/avif", //some files may cause problesms see fox.profile0.8bpc.yuv420.odd-width.odd-height.avif
    "image/bmp",
    "image/vnd.ms-dds",
    "image/gif",
    "image/vnd.microsoft.icon",
    "image/jpeg",
    "image/x-exr",
    "image/png",
    "image/x-portable-bitmap",
    "image/x-portable-anymap",
    "image/x-targa",
    "image/x-tga",
    "image/tiff",
    "image/webp",
    // TODOS
];

pub fn get_type(mime: &str) -> MediaType {
    if SUPPORTED_FORMATS_IMAGE.contains(&mime) {
        return MediaType::Image;
    } else {
        return MediaType::Unknown;
    }
}
