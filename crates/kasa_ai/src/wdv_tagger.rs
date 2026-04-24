use std::iter::zip;

use image::{Rgba, imageops};

use anyhow::Result;
use ort::tensor::TensorElementType::Float32;
use ort::value::Tensor;
use ort::session::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct TaggerOutput {
    pub character: Vec<TaggerTag>,
    pub general: Vec<TaggerTag>,
    pub ratings: TaggerTag,
}

impl TaggerOutput {
    pub fn count(&self) -> i64 {
        (self.character.len() + self.general.len() + 1) as i64
    }
}

#[derive(Debug)]
pub struct TaggerTag {
    pub name: String,
    pub confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaggerThresholds {
    character: f32,
    general: f32,
}

impl Default for TaggerThresholds {
    fn default() -> Self {
        // values from reference implementation
        // https://github.com/SmilingWolf/wdv3-jax/blob/a7f1f6c9fc2d31eaaf327b9168e86d81d7d3e455/wdv3_jax.py#L206
        // where did the onnx implementation go?
        Self {
            character: 0.75,
            general: 0.35,
        }
    }
}

pub fn tag_image_wdv(
    session: &mut Session,
    image_path: &str,
    tag_labels: &Labels,
    thresholds: &TaggerThresholds,
) -> Result<TaggerOutput> {
    let (dim_x, dim_y) = match &session.inputs().first().unwrap().dtype() {
        ort::value::ValueType::Tensor {
            ty,
            shape: dimensions,
            dimension_symbols: _,
        } => {
            assert_eq!(ty, &Float32, "Model not supported");

            let dims = dimensions;
            (dims[1], dims[2])
        }
        _ => unimplemented!("Model not supported"),
    };

    let input_dims = (dim_x as u32, dim_y as u32);

    let image: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 4]>> =
        prepare_image(image_path, input_dims)?;

    // any way to not use ndarray for this?

    let outputs = session.run(ort::inputs!["input" => Tensor::from_array(image)?])?;

    let (_, flattened) = outputs["output"].try_extract_tensor::<f32>()?;

    let tags = tag_labels;
    let labels: Vec<(String, f32)> = zip(tags.tag_names.to_owned(), flattened.to_owned()).collect();

    let ratings: Vec<TaggerTag> = tags
        .rating_indexes
        .iter()
        .map(|i| {
            let (name, force) = &labels[*i];

            TaggerTag {
                name: name.to_owned(),
                confidence: force.to_owned(),
            }
        })
        .collect();

    let general: Vec<TaggerTag> = tags
        .general_indexes
        .iter()
        .map(|i| {
            let (name, force) = &labels[*i];

            TaggerTag {
                name: name.to_owned(),
                confidence: force.to_owned(),
            }
        })
        .collect();

    let character: Vec<TaggerTag> = tags
        .character_indexes
        .iter()
        .map(|i| {
            let (name, force) = &labels[*i];

            TaggerTag {
                name: name.to_owned(),
                confidence: force.to_owned(),
            }
        })
        .collect();

    let mut general: Vec<TaggerTag> = general
        .into_iter()
        .filter(|t| t.confidence > thresholds.general)
        .collect();

    let mut character: Vec<TaggerTag> = character
        .into_iter()
        .filter(|t| t.confidence > thresholds.character)
        .collect();

    let rating = ratings
        .into_iter()
        .max_by(|x, y| {
            x.confidence
                .partial_cmp(&y.confidence)
                .expect("Cannot compare floats?")
        })
        .expect("Cannot compare floats?");

    general.sort_by(|a, b| {
        a.confidence
            .partial_cmp(&b.confidence)
            .expect("Cannot compare floats?")
    });
    character.sort_by(|a, b| {
        a.confidence
            .partial_cmp(&b.confidence)
            .expect("Cannot compare floats?")
    });

    general.reverse();
    character.reverse();

    Ok(TaggerOutput {
        character,
        general,
        ratings: rating,
    })
}

pub struct Labels {
    tag_names: Vec<String>,
    rating_indexes: Vec<usize>,
    general_indexes: Vec<usize>,
    character_indexes: Vec<usize>,
}

pub fn prepare_labels(labels_path: &str) -> Result<Labels> {
    let mut csv = csv::Reader::from_path(labels_path)?;

    let records: Result<Vec<_>, _> = csv.records().collect();
    let records = records?;

    // collect the indexes of tags with the category "rating"
    let rating_indexes: Vec<usize> = records
        .iter()
        .enumerate()
        .filter(|(_, r)| r[2] == *"9")
        .map(|(i, _)| i)
        .collect();

    let general_indexes: Vec<usize> = records
        .iter()
        .enumerate()
        .filter(|(_, r)| r[2] == *"0")
        .map(|(i, _)| i)
        .collect();

    let character_indexes: Vec<usize> = records
        .iter()
        .enumerate()
        .filter(|(_, r)| r[2] == *"4")
        .map(|(i, _)| i)
        .collect();

    Ok(Labels {
        tag_names: records.into_iter().map(|r| r[1].to_string()).collect(),
        rating_indexes,
        general_indexes,
        character_indexes,
    })
}

fn prepare_image(path: &str, size: (u32, u32)) -> Result<ndarray::Array4<f32>> {
    let mut canvas = image::RgbaImage::new(size.0, size.1);
    for p in canvas.pixels_mut() {
        *p = Rgba([255, 255, 255, 255]);
    }

    let inp = image::open(path)?;

    let (inp_target_x, inp_target_y) =
        calculate_aspect_ratio(inp.width(), inp.height(), size.0, size.1);

    let resized = imageops::resize(
        &inp,
        inp_target_x,
        inp_target_y,
        imageops::FilterType::CatmullRom,
    );

    let canvas_width = canvas.width();
    let canvas_height = canvas.height();

    imageops::overlay(
        &mut canvas,
        &resized,
        ((canvas_width - resized.width()) / 2).into(),
        ((canvas_height - resized.height()) / 2).into(),
    );

    let mut converted = ndarray::Array4::<f32>::zeros((1, 448, 448, 3));
    for y in 0..canvas.height() {
        for x in 0..canvas.width() {
            let pixel = canvas.get_pixel(x, y);

            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;

            // Store in NHWC format (batch, height, width, channels)
            converted[[0, y as usize, x as usize, 0]] = b; // Blue channel
            converted[[0, y as usize, x as usize, 1]] = g; // Green channel
            converted[[0, y as usize, x as usize, 2]] = r; // Red channel
        }
    }

    Ok(converted)
}

/// https://stackoverflow.com/a/14731922
/// Conserve aspect ratio of the original region. Useful when shrinking/enlarging
//  images to fit into a certain area.
pub fn calculate_aspect_ratio(
    src_x: u32,
    src_y: u32,
    dest_max_x: u32,
    dest_max_y: u32,
) -> (u32, u32) {
    let ratio = f64::min(
        dest_max_x as f64 / src_x as f64,
        dest_max_y as f64 / src_y as f64,
    );
    ((src_x as f64 * ratio) as u32, (src_y as f64 * ratio) as u32)
}
