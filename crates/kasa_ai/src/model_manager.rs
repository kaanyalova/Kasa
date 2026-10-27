mod image_embedding;
mod ocr;
mod text_embedding;
pub mod wdv_tagger;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use fastembed::{ImageEmbedding, TextEmbedding};
use ort::ep;
use thiserror::Error;

use crate::model_sources::{ModelDownloadProgress, ModelSources, ModelSourcesConfig};

pub struct ModelManagerModels {
    pub model_sources: ModelSources,
    pub models_base_path: PathBuf,
    pub reqwest: reqwest::Client,
    pub on_progress: Box<dyn Fn(ModelDownloadProgress) + Send + Sync>,
}

pub struct ModelManager {
    onnx_rt_dynamic_lib_path: PathBuf,
    is_onnx_rt_loaded: bool,
    pub models: ModelManagerModels,

    // the other models are going to process large batches so they can just be
    // loaded when they are running, but the embedding models should be loaded
    // so the user can make searches without waiting for the model to load
    loaded_text_embedding_model: HashMap<String, TextEmbedding>,
    loaded_image_embedding_model: HashMap<String, ImageEmbedding>,
}

impl ModelManager {
    pub fn new(
        rt_path: &Path,
        models_path: &Path,
        on_progress: Box<dyn Fn(ModelDownloadProgress) + Send + Sync>,
    ) -> Self {
        Self {
            onnx_rt_dynamic_lib_path: rt_path.to_owned(),
            is_onnx_rt_loaded: false,
            models: ModelManagerModels {
                model_sources: ModelSources::default(),
                models_base_path: models_path.to_owned(),
                reqwest: reqwest::Client::new(),
                on_progress,
            },
            loaded_text_embedding_model: HashMap::new(),
            loaded_image_embedding_model: HashMap::new(),
        }
    }

    pub fn load_sources_from_config_file(&mut self, path: &Path) -> Result<()> {
        let string = fs::read_to_string(path)?;
        let config: ModelSourcesConfig = toml::from_str(&string)?;
        self.models.model_sources = config.map_to_names();

        Ok(())
    }

    pub fn load_default_sources(&mut self) -> Result<()> {
        let config: ModelSourcesConfig = toml::from_str(include_str!("./model_sources.toml"))?;
        self.models.model_sources = config.map_to_names();
        Ok(())
    }

    pub fn load_onnx_rt(&mut self) -> Result<()> {
        let ort = ort::init_from(&self.onnx_rt_dynamic_lib_path)?
            .with_execution_providers([
                ep::CUDA::default().build(),
                ep::TensorRT::default().build(),
                ep::MIGraphX::default().build(),
                ep::CPU::default().build(),
            ])
            .commit();

        self.is_onnx_rt_loaded = true;

        Ok(())
    }

    pub fn load_model_sources(&mut self, model_sources: ModelSourcesConfig) {
        self.models.model_sources = model_sources.map_to_names();
    }
}

#[tokio::test]
async fn test_ocr() {
    let mut model_manager = ModelManager::new(
        Path::new(
            "/home/kaan/Belgeler/0000_Projects/Kasa/__dev_models/onnxruntime-linux-x64-1.29.0/lib/libonnxruntime.so",
        ),
        Path::new("/home/kaan/Belgeler/0000_Projects/Kasa/__dev_models"),
        Box::new(|progress: ModelDownloadProgress| {
            println!("{progress:?}");
        }),
    );

    model_manager
        .load_sources_from_config_file(Path::new(
            "/home/kaan/Belgeler/0000_Projects/Kasa/crates/kasa_ai/src/model_sources.toml",
        ))
        .unwrap();

    model_manager.load_onnx_rt().unwrap();

    let model_source = model_manager
        .models
        .model_sources
        .ocr_models
        .get("PP-OcrV6-Small")
        .unwrap()
        .clone();

    let mut model = model_source.into_model(&model_manager.models.models_base_path);

    let client = reqwest::ClientBuilder::new().build().unwrap();

    let images = vec![Path::new("/home/kaan/İndirilenler/1709532341301885.png")];
    let results = model_manager.ocr_images("PP-OcrV6-Small", images).unwrap();
}

#[derive(Debug, Error)]
enum ModelError {
    #[error("The model {0} does not exist in the model sources")]
    ModelDoesNotExistInSources(String),
}
