use anyhow::Result;
use core::str;
use exif::{In, Tag};
use nom::{
    bytes::complete::{tag, take},
    multi::many0,
    IResult,
};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::{fs::File, io::BufReader, path::PathBuf};

use super::prompt_parser::parse_prompt;
use super::SlopTag;
use super::SlopTags;
use super::{
    errors::SlopTagParseError,
    supported_formats::{parse_png_meta, SLOP_SUPPORTED_FORMATS},
};

/// Parses tags from A1111 generated images
#[allow(unused)]
#[deprecated]
fn parse_a1111_tags(path: PathBuf, max_tag_len: usize) -> Result<SlopTags> {
    let mime = mime_guess::from_path(&path).first();

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

            let params = meta.get("parameters");

            let params = match params {
                Some(params) => params,
                None => return Err(SlopTagParseError::A1111ParametersNotFound.into()),
            };

            return Ok(parse_a111_tags_from_meta(&params, max_tag_len)?);
        }

        // rest of the image formats that use exif data
        _ => {
            let file = File::open(path)?;
            let mut bufreader = BufReader::new(&file);
            let exif_reader = exif::Reader::new();

            let exif = exif_reader.read_from_container(&mut bufreader)?;

            let user_comment = exif.get_field(Tag::UserComment, In::PRIMARY);

            let user_comment = match user_comment {
                Some(user_comment) => user_comment,
                None => return Err(SlopTagParseError::ImageDoesntHaveUserCommentExif.into()),
            };

            //let user_comment_contents = user_comment.display_value().with_unit(&exif).to_string();
            let user_comment_val = &user_comment.value;

            let user_comment_bytes = match user_comment_val {
                exif::Value::Undefined(bytes, _) => bytes,
                _ => {
                    return Err(SlopTagParseError::InvalidUserComment.into());
                }
            }
            .to_owned();

            // https://stackoverflow.com/questions/55184864/nom-parser-borrow-checker-issue
            let (_, user_comment) = parse_a111_jpeg_usercomment(&user_comment_bytes)
                .map_err(|e| SlopTagParseError::NomParserError(e.to_string()))?;

            // I have no idea what this is, but some random image i downloaded from civit has it
            // might as well strip it

            let user_comment = user_comment
                .strip_prefix("detailxl.")
                .unwrap_or(&user_comment);

            let tags = parse_a111_tags_from_meta(user_comment, max_tag_len)?;

            Ok(tags)
        }
    }
}

pub fn parse_a111_tags_from_meta(params: &str, max_tag_len: usize) -> Result<SlopTags> {
    // parse params until newline

    let split = params.split_once("Steps:");
    let prompts = match split {
        Some((prompts, _)) => prompts,
        None => return Err(SlopTagParseError::A1111NoPromptEndFound.into()),
    };

    // if the "Negative prompt: " string does't exist it is likely the image doesn't have a negative prompt, so return the full prompt
    let (positive_prompts, negative_prompts) = prompts
        .split_once("Negative prompt:")
        .map(|(pos, neg)| (pos, Some(neg)))
        .unwrap_or((prompts, None));

    let tags_positive = prompt_to_tags(positive_prompts, max_tag_len);

    let tags_negative = if let Some(negs) = negative_prompts {
        prompt_to_tags(negs, max_tag_len)
    } else {
        vec![]
    };

    let tags = SlopTags {
        positive: tags_positive,
        negative: tags_negative,
    };

    Ok(tags)
}

/// Converts prompts text to vector of tags, does various types of filtering
fn prompt_to_tags(input: &str, max_tag_len: usize) -> Vec<SlopTag> {
    let inputs = input.replace(r#"\("#, "LBĞ").replace(r#"\)"#, "RBĞ");

    parse_prompt(&inputs)
        .into_par_iter()
        .map(|t| {
            let name = t
                .name
                .trim()
                .to_string()
                .replace("LBĞ", "(")
                .replace("RBĞ", ")")
                .replace(" ", "_");

            SlopTag {
                name,
                power: t.power,
            }
        })
        .filter(|t| t.name.to_lowercase() != "break")
        .filter(|t| t.name.len() < max_tag_len)
        .collect()
}

fn parse_zero_padded_byte(input: &[u8]) -> IResult<&[u8], u8> {
    let (input, _) = tag([0x0])(input)?;
    let (input, output) = take(1usize)(input)?;
    Ok((input, output[0]))
}

/// Parses the weird 0 padded format used by A1111
pub fn parse_a111_jpeg_usercomment(input: &[u8]) -> IResult<&[u8], String> {
    let (input, _) = tag([0x55, 0x4E, 0x49, 0x43, 0x4F, 0x44, 0x45, 0x00])(input)?; // UNICODE\0
    let (_input, output) = many0(parse_zero_padded_byte)(input)?;
    let string = String::from_utf8_lossy(&output).to_string();

    Ok((input, string))
}

#[test]
fn a1111_png_meta_parsing() {
    #[allow(deprecated)]
    let tags = parse_a1111_tags(
        "src/ai_slop/test_assets/a1111_example.png".into(),
        usize::MAX,
    )
    .unwrap();

    #[allow(unused)]
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

    dbg!(tags);
}

#[test]
fn a1111_jpeg_meta_parsing() {
    #[allow(deprecated)]
    let tags = parse_a1111_tags("src/ai_slop/test_assets/a1111_example.jpeg".into(), 30).unwrap();

    let expected_json =
        std::fs::read_to_string("src/ai_slop/test_assets/a1111_meta_expected_jpeg.json").unwrap();

    let expected: SlopTags = serde_json::from_str(&expected_json).unwrap();

    assert_eq!(tags, expected)
}
