use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use fastembed::{
    ImageEmbedding, ImageInitOptionsUserDefined, InitOptionsUserDefined, TextEmbedding,
    TokenizerFiles, UserDefinedEmbeddingModel, UserDefinedImageEmbeddingModel,
};
use oar_ocr::{
    core::ModelSource,
    domain::structure::LayoutElementType::Region,
    oarocr::{OAROCRBuilder, OAROCRResult, TextRegion},
    utils::load_image,
};
use ort::{ep, session::Session};
use thiserror::Error;

use crate::model_sources::{
    self, ImageEmbeddingModel, Model, ModelDownloadProgress, ModelSources, ModelSourcesConfig,
    ModelState, OcrModel, TextEmbeddingModel,
};

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
    // loaded when they are running, but the text embedding models should be loaded
    // so the user can make searches without waiting for the model to load
    loaded_text_embedding_model: HashMap<String, TextEmbedding>,
}

static OCR_IMAGE_BATCH_COUNT: usize = 10;
static EMBEDDING_IMAGE_BATCH_COUNT: usize = 10;

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

    /// returns the vector of text regions while keeping the original indices of the input image_paths
    /// if something goes wrong while ocring the image it returns None for that index
    pub fn ocr_images(
        &self,
        model_name: &str,
        image_paths: Vec<&Path>,
    ) -> Result<Vec<Option<Vec<TextRegion>>>> {
        assert!(self.is_onnx_rt_loaded);
        let model = self
            .models
            .get_downloaded_ocr_model(model_name)
            .context(format!("the model with name {} doesn't exist", model_name))?;

        let paths = &model.paths;
        let ocr = OAROCRBuilder::new(
            &paths.det_model_path,
            &paths.rec_model_path,
            &paths.dict_path,
        )
        .build()?;

        let mut all_results = Vec::with_capacity(image_paths.len());

        for image_batch in image_paths.chunks(OCR_IMAGE_BATCH_COUNT) {
            let (indices_of_original_array, valid_images): (Vec<_>, Vec<_>) = image_batch
                .iter()
                .enumerate()
                .filter(|(_i, p)| p.exists())
                .filter_map(|(i, p)| load_image(p).ok().map(|img| (i, img)))
                .unzip();

            let results = ocr.predict(valid_images)?;

            let mut mapped_results = vec![None; OCR_IMAGE_BATCH_COUNT];

            // map them back to a new array so it keeps the original indices and invalid images are None
            for (orig_idx, result) in indices_of_original_array.into_iter().zip(results) {
                mapped_results[orig_idx] = Some(result);
            }

            all_results.extend(mapped_results);
        }

        // lets just serialize whatever oarocr outputs later, it derives serialize
        let text_regions: Vec<Option<Vec<TextRegion>>> = all_results
            .into_iter()
            .map(|r| r.map(|r| r.text_regions))
            .collect();

        Ok(text_regions)
    }

    pub fn calculate_embeddings_for_images(
        &self,
        model_name: &str,
        image_paths: Vec<&Path>,
    ) -> Result<Vec<Option<Vec<f32>>>> {
        let embedding_model = self
            .models
            .get_downloaded_image_embedding_model(model_name)
            .context(format!("the model with name {} doesn't exist", model_name))?;

        let model_bytes = fs::read(embedding_model.paths.model)?;
        let preprocessor_config_bytes = fs::read(embedding_model.paths.preprocessor_config)?;

        let mut embedding = ImageEmbedding::try_new_from_user_defined(
            UserDefinedImageEmbeddingModel::new(model_bytes, preprocessor_config_bytes),
            ImageInitOptionsUserDefined::new(),
        )?;

        let mut all_results = Vec::with_capacity(image_paths.len());

        for image_batch in image_paths.chunks(EMBEDDING_IMAGE_BATCH_COUNT) {
            let (indices_of_original_array, valid_images): (Vec<_>, Vec<_>) = image_batch
                .iter()
                .enumerate()
                .filter(|(_i, p)| p.exists())
                .filter_map(|(i, p)| image::open(p).ok().map(|img| (i, img)))
                .unzip();

            let results = embedding.embed_images(valid_images)?;

            let mut mapped_results = vec![None; EMBEDDING_IMAGE_BATCH_COUNT];

            // map them back to a new array so it keeps the original indices and invalid images are None
            for (orig_idx, result) in indices_of_original_array.into_iter().zip(results) {
                mapped_results[orig_idx] = Some(result);
            }

            all_results.extend(mapped_results);
        }

        Ok(all_results)
    }

    /// unlike the other functions this one runs on loaded models instead
    pub fn calculate_embeddings_for_text(
        &mut self,
        model_name: &str,
        text: &str,
    ) -> Result<Vec<f32>> {
        let loaded_model = self
            .loaded_text_embedding_model
            .get_mut(model_name)
            .context(format!(
                "the text embedding model {} was not loaded in the memory",
                model_name
            ))?;

        let texts = vec![text];
        let embedded = loaded_model.embed(texts, Some(1))?.remove(0);

        Ok(embedded)
    }

    pub fn load_text_embedding_model_to_memory(&mut self, name: &str) -> Result<()> {
        let model = self
            .models
            .get_downloaded_text_embedding_model("name")
            .context(format!("model with name {} not found", name))?;

        let model_bytes = fs::read(model.paths.model)?;
        let tokenizer_bytes = fs::read(model.paths.tokenizer_file)?;
        let config_file_bytes = fs::read(model.paths.config_file)?;
        let special_token_map_bytes = fs::read(model.paths.special_tokens_map_file)?;
        let tokenizer_config_bytes = fs::read(model.paths.tokenizer_config_file)?;

        let loaded = TextEmbedding::try_new_from_user_defined(
            UserDefinedEmbeddingModel::new(
                model_bytes,
                TokenizerFiles {
                    tokenizer_file: tokenizer_bytes,
                    config_file: config_file_bytes,
                    special_tokens_map_file: special_token_map_bytes,
                    tokenizer_config_file: tokenizer_config_bytes,
                },
            ),
            InitOptionsUserDefined::new(),
        )?;

        self.loaded_text_embedding_model
            .insert(name.to_owned(), loaded);

        Ok(())
    }

    pub fn drop_loaded_embedding_models(&mut self) {
        self.loaded_text_embedding_model.clear();
    }
}

impl ModelManagerModels {
    pub async fn download_ocr_model(&mut self, name: &str) -> Result<()> {
        let model_source = self
            .model_sources
            .ocr_models
            .get(name)
            .context(format!("the model with name {} doesn't exist", name))?;

        let mut model = model_source.clone().into_model(&self.models_base_path);

        model
            .download_model(&self.reqwest, self.on_progress.as_ref())
            .await?;

        Ok(())
    }

    pub fn get_downloaded_ocr_model(&self, name: &str) -> Option<OcrModel> {
        let model_source = self.model_sources.ocr_models.get(name)?;

        let mut model = model_source.clone().into_model(&self.models_base_path);

        if model.check_if_already_downloaded() {
            Some(model)
        } else {
            None
        }
    }

    pub async fn download_image_embedding_model(&mut self, name: &str) -> Result<()> {
        let model_source = self
            .model_sources
            .image_embedding_models
            .get(name)
            .context(format!("the model with name {} doesn't exist", name))?;

        let mut model = model_source.clone().into_model(&self.models_base_path);

        model
            .download_model(&self.reqwest, self.on_progress.as_ref())
            .await?;

        Ok(())
    }

    pub fn get_downloaded_image_embedding_model(&self, name: &str) -> Option<ImageEmbeddingModel> {
        let model_source = self.model_sources.image_embedding_models.get(name)?;
        let mut model = model_source.clone().into_model(&self.models_base_path);

        if model.check_if_already_downloaded() {
            Some(model)
        } else {
            None
        }
    }
    pub fn is_ocr_model_downloaded(&self, name: &str) -> bool {
        let has_source = self.model_sources.ocr_models.contains_key(name);

        if has_source {
            let source = self.model_sources.ocr_models.get(name).unwrap();
        }

        todo!()
    }

    pub async fn download_text_embedding_model(&mut self, name: &str) -> Result<()> {
        let model_source = self
            .model_sources
            .text_embedding_models
            .get(name)
            .context(format!("the model with name {} doesn't exist", name))?;

        let mut model = model_source.clone().into_model(&self.models_base_path);

        model
            .download_model(&self.reqwest, self.on_progress.as_ref())
            .await?;

        Ok(())
    }

    pub fn get_downloaded_text_embedding_model(&self, name: &str) -> Option<TextEmbeddingModel> {
        let model_source = self.model_sources.text_embedding_models.get(name)?;

        let mut model = model_source.clone().into_model(&self.models_base_path);

        if model.check_if_already_downloaded() {
            Some(model)
        } else {
            None
        }
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
