use crate::db::schema::MediaType;

// https://developer.mozilla.org/en-US/docs/Web/HTTP/MIME_types/Common_types
pub const SUPPORTED_FORMATS: [&str; 22] = [
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
    "video/x-msvideo", // avi
    "video/mp4",
    "video/mpeg",
    "video/ogg",
    "video/mp2t",
    "video/webm",
    "video/3gpp",
    "video/3g2",
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

pub const SUPPORTED_FORMATS_VIDEO: [&str; 8] = [
    "video/x-msvideo", // avi
    "video/mp4",
    "video/mpeg",
    "video/ogg",
    "video/mp2t",
    "video/webm",
    "video/3gpp",
    "video/3g2",
];

pub fn get_type(mime: &str) -> MediaType {
    // TODO replace this
    if SUPPORTED_FORMATS_IMAGE.contains(&mime) {
        return MediaType::Image;
    } else if SUPPORTED_FORMATS_VIDEO.contains(&mime) {
        return MediaType::Video;
    } else {
        return MediaType::Unknown;
    }
}
