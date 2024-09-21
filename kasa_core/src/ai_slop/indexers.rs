use anyhow::{Ok, Result};
use exif::{In, Tag};
use itertools::Itertools;
use nom::Finish;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sqlx::{Execute, Pool, QueryBuilder, Sqlite};
use std::{fs::File, io::BufReader, path::PathBuf, usize};

use crate::{
    ai_slop::comfy::ComfyExifJson,
    db::{self},
    tags::tags::insert_tags,
};

use super::{
    a1111::{parse_a111_jpeg_usercomment, parse_a111_tags_from_meta},
    comfy::{parse_comfy_tags_from_meta, ComfyPrompt},
    errors::SlopTagParseError,
    supported_formats::{parse_png_meta, SLOP_SUPPORTED_FORMATS},
    SlopTag, SlopTags,
};

/// Gets AI metadata from various image types for A1111 and Comfy metadata
pub fn extract_meta_from_img(path: &PathBuf) -> Result<SlopImageMeta> {
    let mime = mime_guess::from_path(path).first();

    let mime = match mime {
        Some(mime) => {
            if !SLOP_SUPPORTED_FORMATS.contains(&mime.to_string().as_ref()) {
                return Err(SlopTagParseError::UnsupportedFile(mime.to_string()).into());
            }
            mime
        }
        None => return Err(SlopTagParseError::UnsupportedFile("Unknown".to_string()).into()),
    };

    match mime.to_string().as_ref() {
        "image/png" => {
            let meta = parse_png_meta(&path)?;

            // A1111 png images have the `parameters` field on them
            if let Some(meta_val) = meta.get("parameters") {
                return Ok(SlopImageMeta::A1111Png(meta_val.to_owned()));
            }

            // Comfy png images have the `prompt` field on them
            if let Some(meta_val) = meta.get("prompt") {
                return Ok(SlopImageMeta::ComfyPng(meta_val.to_owned()));
            }

            return Ok(SlopImageMeta::NoMeta);
        }

        // Other supported image formats that use UserComment
        _ => {
            let file = File::open(path)?;
            let mut bufreader = BufReader::new(&file);
            let exif_reader = exif::Reader::new();

            let exif = exif_reader.read_from_container(&mut bufreader)?;
            //for f in exif.fields() {
            //    dbg!(f);
            //}

            let user_comment = {
                if let Some(user_comment) = exif.get_field(Tag::UserComment, In::PRIMARY) {
                    user_comment
                } else if let Some(user_comment) =
                    exif.get_field(Tag(exif::Context::Tiff, 37510), In::PRIMARY)
                {
                    user_comment
                } else {
                    return Ok(SlopImageMeta::NoMeta);
                }
            };

            //exif.get_field(Tag::UserComment, In::PRIMARY);

            let user_comment_val = &user_comment.value;

            let user_comment_bytes = match user_comment_val {
                exif::Value::Undefined(bytes, _) => bytes,
                exif::Value::Ascii(bytes) => &bytes[0],
                _ => {
                    return Err(SlopTagParseError::InvalidUserComment.into());
                }
            }
            .to_owned();

            // A1111 images have `S\0t\0\e\0\p\0\s\0: (Steps:)` on their exif data
            // TODO? There might be a faster way of determining if the image comes from A111
            let needle: [u8; 0x0B] = [
                0x53, 0x00, 0x74, 0x00, 0x65, 0x00, 0x70, 0x00, 0x73, 0x00, 0x3A,
            ];

            // cannot believe i need a crate for this
            let is_a1111 = memchr::memmem::find(&user_comment_bytes, &needle).is_some();

            if is_a1111 {
                return Ok(SlopImageMeta::A1111Other(user_comment_bytes));
            }
            if user_comment_bytes.starts_with(r#"{"prompt":"#.as_bytes()) {
                return Ok(SlopImageMeta::ComfyOther(user_comment_bytes));
            }

            Ok(SlopImageMeta::NoMeta)
        }
    }
}

/// The main function that extracts AI generated prompts from images
/// Supports A1111 and ComfyUI  
pub fn get_prompt_tags_from_img(path: &PathBuf, max_tag_len: usize) -> Result<Option<SlopTags>> {
    match extract_meta_from_img(path)? {
        SlopImageMeta::A1111Png(meta) => {
            let tags = parse_a111_tags_from_meta(&meta, max_tag_len)?;
            return Ok(Some(tags));
        }
        SlopImageMeta::A1111Other(meta) => {
            let (_, meta) = parse_a111_jpeg_usercomment(&meta)
                .finish()
                .map_err(|_| SlopTagParseError::NomParserError("Zero encoded A1111 exif parser failed, even though the image was detected to be an image generated using A1111".to_string()))?;

            let tags = parse_a111_tags_from_meta(&meta, max_tag_len)?;

            return Ok(Some(tags));
        }
        SlopImageMeta::ComfyPng(meta) => {
            //println!("{}", String::from_utf8(meta_b));

            let parsed: ComfyPrompt = serde_json::from_str(&meta)?;
            Ok(Some(parse_comfy_tags_from_meta(&parsed)))
        }
        SlopImageMeta::ComfyOther(meta_b) => {
            let meta = unsafe { String::from_utf8_unchecked(meta_b.clone()) }; // todo remove unsafe
            let meta = meta.replace("\n", "");

            let json: ComfyExifJson = serde_json::from_str(&meta)?;

            Ok(Some(parse_comfy_tags_from_meta(&json.prompt)))
        }
        SlopImageMeta::NoMeta => return Ok(None),
    }
}

const SLOP_PROMPT_PATHS_DB_LIMIT: usize = 1000;

/// Run after an index
pub async fn get_prompt_tags_from_ids_batch(
    input_ids: Vec<String>,
    max_tag_len: usize,
    pool: &Pool<Sqlite>,
) {
    if input_ids.len() == 0 {
        return;
    }

    for ids_batch in &input_ids.into_iter().chunks(SLOP_PROMPT_PATHS_DB_LIMIT) {
        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT * FROM Path WHERE hash IN (");

        let mut separated = query_builder.separated(", ");
        for hash in ids_batch {
            separated.push_bind(hash);
        }

        separated.push_unseparated(") ");

        //let sql = query_builder.into_sql();
        //dbg!(sql);

        let paths_query = query_builder.build_query_as::<db::schema::Path>();
        let s = paths_query.sql();

        //dbg!(s);

        let paths = paths_query.fetch_all(pool).await.unwrap();

        let sloptag_vecs: Vec<ImageTagReferences> = paths
            .into_par_iter()
            .map(|_path| {
                (
                    get_prompt_tags_from_img(&PathBuf::from(_path.path), max_tag_len),
                    _path.hash,
                )
            })
            .filter(|(t, h)| t.is_ok())
            .map(|(t, h)| (t.unwrap(), h))
            .filter(|(t, h)| t.is_some())
            .map(|(t, h)| {
                let pos: Vec<String> = t.unwrap().positive.into_iter().map(|t| t.name).collect();
                ImageTagReferences { hash: h, tags: pos }
            })
            .collect();

        for t in sloptag_vecs {
            insert_tags(t.tags, pool, Some(t.hash)).await;
        }
    }
}

#[derive(Debug)]
struct ImageTagReferences {
    pub hash: String,
    pub tags: Vec<String>,
}

pub enum SlopImageMeta {
    /// `parameters` field of the png
    A1111Png(String),
    /// `UserComment` field of exif data, which is weirdly encoded with \0 for some reason?
    A1111Other(Vec<u8>),
    /// `prompt` field of the png
    ComfyPng(String),
    /// `prompt` field of UserComment which is inside of a nested json structure
    /// https://github.com/kaanyalova/ComfyUI_ExtendedImageFormats
    ComfyOther(Vec<u8>),
    /// No AI metadata was found
    NoMeta,
}

#[test]
fn test_a1111_tags_png() {
    let tags = get_prompt_tags_from_img(
        &"src/ai_slop/test_assets/a1111_example.png".into(),
        usize::MAX,
    )
    .unwrap()
    .unwrap();

    let expected = SlopTags {
        positive: vec![
            SlopTag {
                name: "Astronaut_in_a_jungle".to_string(),
                power: 1.0,
            },
            SlopTag {
                name: "cold_color_palette".to_string(),
                power: 1.0,
            },
            SlopTag {
                name: "muted_colors".to_string(),
                power: 1.0,
            },
            SlopTag {
                name: "detailed".to_string(),
                power: 1.0,
            },
            SlopTag {
                name: "8k".to_string(),
                power: 1.0,
            },
        ],
        negative: vec![],
    };

    assert_eq!(expected, tags);
}

#[test]
fn test_a1111_tags_jpeg() {
    let tags = get_prompt_tags_from_img(&"src/ai_slop/test_assets/a1111_example.jpeg".into(), 30)
        .unwrap()
        .unwrap();

    let expected_file =
        std::fs::read_to_string("src/ai_slop/test_assets/a1111_meta_expected_jpeg.json").unwrap();
    let expected: SlopTags = serde_json::from_str(&expected_file).unwrap();

    assert_eq!(expected, tags)
}

#[test]
fn test_comfy_tags_png() {
    let tags = get_prompt_tags_from_img(
        &"src/ai_slop/test_assets/comfy_example.png".into(),
        usize::MAX,
    )
    .unwrap()
    .unwrap();

    let expected = SlopTags {
        positive: vec![
            SlopTag {
                name: "evening sunset scenery blue sky nature".to_string(),
                power: 1.0,
            },
            SlopTag {
                name: "glass bottle with a galaxy in it".to_string(),
                power: 1.0,
            },
        ],
        negative: vec![
            SlopTag {
                name: "watermark".to_string(),
                power: 1.0,
            },
            SlopTag {
                name: "text".to_string(),
                power: 1.0,
            },
        ],
    };

    assert_eq!(tags, expected)
}
