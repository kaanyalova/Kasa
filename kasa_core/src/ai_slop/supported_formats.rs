use anyhow::{Ok, Result};
use std::{collections::HashMap, hash::Hash, path::PathBuf};

use image::EncodableLayout;
use img_parts::png::{Png, PngChunk};

use crate::db;

use super::errors::SlopTagParseError;

pub const SLOP_SUPPORTED_FORMATS: [&str; 4] =
    ["image/png", "image/jpeg", "image/webp", "image/avif"];

pub fn parse_png_meta(path: &PathBuf) -> Result<HashMap<String, String>> {
    let file = std::fs::read(path)?;
    let png = Png::from_bytes(file.into())?;

    let chunks = png.chunks_by_type([0x74, 0x45, 0x58, 0x74]); // tEXt field of PNGs

    let strings: Result<HashMap<String, String>> = chunks
        .map(|chunk| {
            let contents = chunk.contents();
            let string = String::from_utf8_lossy(contents.as_bytes()).to_string();

            let split: Vec<&str> = string.split("\0").collect();

            // make sure that there is only one separator
            if split.len() != 2 {
                return Err(SlopTagParseError::PngTextParseSplitError.into());
            }

            Ok((split[0].to_string(), split[1].to_string()))
        })
        .collect();

    Ok(strings?)
}

#[test]
fn png_meta_parsing() {
    let meta_a1111 = parse_png_meta(&"src/ai_slop/test_assets/a1111_example.png".into()).unwrap();

    let mut expected: HashMap<String, String> = HashMap::new();

    expected.insert("parameters".to_string(), "Astronaut in a jungle, cold color palette, muted colors, detailed, 8k\nSteps: 50, Sampler: DPM++ 2M Karras, CFG scale: 5, Seed: 42, Size: 1024x1024, Model hash: 1f69731261, Model: sd_xl_base_0.9, Clip skip: 2, RNG: CPU, Version: v1.4.1-166-g21aec6f5".to_string());

    assert_eq!(expected, meta_a1111);

    let meta_comfy = parse_png_meta(&"src/ai_slop/test_assets/comfy_example.png".into()).unwrap();

    let expected_json =
        std::fs::read_to_string("src/ai_slop/test_assets/comfy_meta_expected.json").unwrap();

    // TODO fix this test!, no need for recursive json, at least for png meta

    //let expected: HashMap<String, String> = serde_json::from_str(&expected_json).unwrap();

    //assert_eq!(meta_comfy, expected);
}
