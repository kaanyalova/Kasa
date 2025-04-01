use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::db::schema::{Image, MediaType};

#[derive(Debug)]
pub struct DbWritableMediaDataBatch {
    pub media_type_identifier: MediaType,
    pub media_data: Vec<MediaTypeWithData>,
    pub generic_media_data: Vec<GenericMediaData>,
    pub paths: Vec<PathData>,
}
#[derive(Debug)]

pub struct PathData {
    pub path: String,
    pub hash: String,
}

#[derive(Debug)]
pub struct GenericMediaData {
    pub hash: String,
    pub size: u64,
    pub mime: String,
    pub thumb_path: Option<String>,
    pub time_added: i64,
    pub thumbnail_x: i64,
    pub thumbnail_y: i64,
}

/*

use db::schema::Image

#[derive(Debug, Clone)]
pub struct ImageData {
    pub hash: String,
    pub resolution_x: u64,
    pub resolution_y: u64,
}
 */
#[derive(Debug, Clone)]
pub enum MediaTypeWithData {
    Image(Image),
    Video, // TODO add metadata
    Invalid(String),
}

// Not a table anymore ,move this later
#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct FirstPass {
    pub hash: String,
    pub path: String,
    pub mime: String,
}
