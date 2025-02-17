use std::iter::zip;

use anyhow::Result;
use ffmpeg::error;
use image::{imageops, Rgba};
use thiserror::Error;

use ort::tensor::TensorElementType::Float32;
use ort::{execution_providers::ROCmExecutionProvider, session::Session};

#[derive(Debug)]
pub struct TaggerOutput {
    pub character: Vec<TaggerTag>,
    pub general: Vec<TaggerTag>,
    pub ratings: TaggerTag,
}

#[derive(Debug)]
pub struct TaggerTag {
    pub name: String,
    pub confidence: f32,
}

pub fn prepare_session(model_path: &str) -> Session {
    std::env::set_var("HSA_OVERRIDE_GFX_VERSION", "10.3.0");
    std::env::set_var("HIP_VISIBLE_DEVICES", "0");
    let onnx_path = std::env::var("KASA_ONNX_RT_PATH").unwrap();
    ort::init_from(&onnx_path)
        .with_execution_providers([ROCmExecutionProvider::default().build().error_on_failure()])
        .commit()
        .unwrap();

    Session::builder()
        .unwrap()
        .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
        .unwrap()
        .with_intra_threads(16)
        .unwrap()
        .commit_from_file(model_path)
        .unwrap()
}

pub fn tag_image_wdv(
    session: &Session,
    image_path: &str,
    tag_labels: &Labels,
    character_thresh: f32,
    general_thresh: f32,
) -> TaggerOutput {
    let (dim_x, dim_y) = match &session.inputs.first().unwrap().input_type {
        ort::value::ValueType::Tensor {
            ty,
            dimensions,
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
        prepare_image(image_path, input_dims);

    // any way to not use ndarray for this?

    let outputs = session
        .run(ort::inputs!["input" => image].unwrap())
        .unwrap();

    let predictions = outputs["output"].try_extract_tensor::<f32>().unwrap();
    let flattened = predictions.flatten();

    let tags = tag_labels;
    let labels: Vec<(String, f32)> = zip(
        tags.tag_names.to_owned().into_iter(),
        flattened.to_owned().into_iter(),
    )
    .collect();

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
        .filter(|t| t.confidence > general_thresh)
        .collect();

    let mut character: Vec<TaggerTag> = character
        .into_iter()
        .filter(|t| t.confidence > character_thresh)
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

    TaggerOutput {
        character: character,
        general: general,
        ratings: rating,
    }
}

pub struct Labels {
    tag_names: Vec<String>,
    rating_indexes: Vec<usize>,
    general_indexes: Vec<usize>,
    character_indexes: Vec<usize>,
}

pub fn prepare_labels(labels_path: &str) -> Labels {
    let mut csv = csv::Reader::from_path(labels_path).unwrap();

    let records: Vec<_> = csv.records().collect();

    // collect the indexes of tags with the category "rating"
    let rating_indexes: Vec<usize> = records
        .iter()
        .enumerate()
        .filter(|(_, r)| r.as_ref().unwrap()[2] == *"9")
        .map(|(i, _)| i.to_owned())
        .collect();

    let general_indexes: Vec<usize> = records
        .iter()
        .enumerate()
        .filter(|(_, r)| r.as_ref().unwrap()[2] == *"0")
        .map(|(i, _)| i)
        .collect();

    let character_indexes: Vec<usize> = records
        .iter()
        .enumerate()
        .filter(|(_, r)| r.as_ref().unwrap()[2] == *"4")
        .map(|(i, _)| i)
        .collect();

    Labels {
        tag_names: records
            .into_iter()
            .map(|r| r.unwrap()[1].to_string())
            .collect(),
        rating_indexes,
        general_indexes,
        character_indexes,
    }
}

fn prepare_image(path: &str, size: (u32, u32)) -> ndarray::Array4<f32> {
    let mut canvas = image::RgbaImage::new(size.0, size.1);
    for p in canvas.pixels_mut() {
        *p = Rgba([255, 255, 255, 255]);
    }

    let inp = image::open(path).unwrap();

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

    return converted;
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
    (
        (src_x as f64 * ratio as f64) as u32,
        (src_y as f64 * ratio as f64) as u32,
    )
}
