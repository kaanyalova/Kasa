use std::{iter::zip, path::Path};

use anyhow::{Context, Result, anyhow};
use image::{Rgba, imageops};
use ort::{
    session::{Session, builder::GraphOptimizationLevel},
    value::{Tensor, TensorElementType::Float32},
};
use serde::{Deserialize, Serialize};

use super::{ModelManager, ModelManagerModels};
use crate::model_sources::{Model, WdvModel};

static WDV_IMAGE_BATCH_COUNT: usize = 10;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

impl ModelManager {
    /// tags the given images, keeping the original indices of the input image_paths
    /// if something goes wrong while tagging an image it returns None for that index
    pub fn tag_images_wdv(
        &self,
        model_name: &str,
        image_paths: Vec<&Path>,
        thresholds: &TaggerThresholds,
    ) -> Result<Vec<Option<TaggerOutput>>> {
        assert!(self.is_onnx_rt_loaded);

        let model = self
            .models
            .get_downloaded_wdv_model(model_name)
            .context(format!("the model with name {} doesn't exist", model_name))?;

        let labels = prepare_labels(&model.paths.tag_labels_path)?;

        let mut session = Session::builder()
            .map_err(ort_error)?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(ort_error)?
            .with_intra_threads(16)
            .map_err(ort_error)?
            .commit_from_file(&model.paths.model_path)
            .map_err(ort_error)?;

        tag_images_wdv(&mut session, image_paths, &labels, thresholds)
    }
}

impl ModelManagerModels {
    pub async fn download_wdv_model(&mut self, name: &str) -> Result<()> {
        let model_source = self
            .model_sources
            .wdv_models
            .get(name)
            .context(format!("the model with name {} doesn't exist", name))?;

        let mut model = model_source.clone().into_model(&self.models_base_path);

        model
            .download_model(&self.reqwest, self.on_progress.as_ref())
            .await?;

        Ok(())
    }

    pub fn get_downloaded_wdv_model(&self, name: &str) -> Option<WdvModel> {
        let model_source = self.model_sources.wdv_models.get(name)?;

        let mut model = model_source.clone().into_model(&self.models_base_path);

        if model.check_if_already_downloaded() {
            Some(model)
        } else {
            None
        }
    }
}

/// runs the tagger over a batch of images, keeping the original indices of the input image_paths
/// if something goes wrong while tagging an image it returns None for that index
pub fn tag_images_wdv(
    session: &mut Session,
    image_paths: Vec<&Path>,
    tag_labels: &Labels,
    thresholds: &TaggerThresholds,
) -> Result<Vec<Option<TaggerOutput>>> {
    let input_dims = input_dimensions(session);

    let mut all_results = Vec::with_capacity(image_paths.len());

    for image_batch in image_paths.chunks(WDV_IMAGE_BATCH_COUNT) {
        let mut mapped_results: Vec<Option<TaggerOutput>> = vec![None; image_batch.len()];

        // map them back to a new array so it keeps the original indices and invalid images are None
        for (orig_idx, path) in image_batch.iter().enumerate() {
            if !path.exists() {
                continue;
            }

            mapped_results[orig_idx] =
                tag_image_with_dims(session, path, tag_labels, thresholds, input_dims).ok();
        }

        all_results.extend(mapped_results);
    }

    Ok(all_results)
}

pub fn tag_image_wdv(
    session: &mut Session,
    image_path: impl AsRef<Path>,
    tag_labels: &Labels,
    thresholds: &TaggerThresholds,
) -> Result<TaggerOutput> {
    let input_dims = input_dimensions(session);
    tag_image_with_dims(session, image_path, tag_labels, thresholds, input_dims)
}

/// same as [tag_image_wdv] but reuses input dimensions that were already read from the session
fn tag_image_with_dims(
    session: &mut Session,
    image_path: impl AsRef<Path>,
    tag_labels: &Labels,
    thresholds: &TaggerThresholds,
    input_dims: (u32, u32),
) -> Result<TaggerOutput> {
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

/// reads the input dimensions the wdv onnx model expects from its session metadata
pub fn input_dimensions(session: &Session) -> (u32, u32) {
    match &session.inputs().first().unwrap().dtype() {
        ort::value::ValueType::Tensor {
            ty,
            shape: dimensions,
            dimension_symbols: _,
        } => {
            assert_eq!(ty, &Float32, "Model not supported");

            let dims = dimensions;
            (dims[1] as u32, dims[2] as u32)
        }
        _ => unimplemented!("Model not supported"),
    }
}

/// `ort::Error` carries the builder it failed on, which is not `Send + Sync`, so it can not be
/// converted into an [anyhow::Error] directly with `?`
fn ort_error<R>(error: ort::Error<R>) -> anyhow::Error {
    anyhow!("onnx runtime error: {error}")
}

pub struct Labels {
    tag_names: Vec<String>,
    rating_indexes: Vec<usize>,
    general_indexes: Vec<usize>,
    character_indexes: Vec<usize>,
}

pub fn prepare_labels(labels_path: impl AsRef<Path>) -> Result<Labels> {
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

fn prepare_image(path: impl AsRef<Path>, size: (u32, u32)) -> Result<ndarray::Array4<f32>> {
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
