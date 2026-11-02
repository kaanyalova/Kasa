use std::{fs, path::Path};

use anyhow::{Context, Result};
use fastembed::{ImageEmbedding, ImageInitOptionsUserDefined, UserDefinedImageEmbeddingModel};

use super::{ModelManager, ModelManagerModels};
use crate::model_sources::{ImageEmbeddingModel, Model};

static EMBEDDING_IMAGE_BATCH_COUNT: usize = 10;

impl ModelManager {
    /// unlike the other functions this one runs on loaded models instead
    pub fn calculate_embeddings_for_images(
        &mut self,
        model_name: &str,
        image_paths: Vec<&Path>,
    ) -> Result<Vec<Option<Vec<f32>>>> {
        let loaded_model = self
            .loaded_image_embedding_model
            .get_mut(model_name)
            .context(format!(
                "the image embedding model {} was not loaded in the memory",
                model_name
            ))?;

        let mut all_results = Vec::with_capacity(image_paths.len());

        for image_batch in image_paths.chunks(EMBEDDING_IMAGE_BATCH_COUNT) {
            let (indices_of_original_array, valid_images): (Vec<_>, Vec<_>) = image_batch
                .iter()
                .enumerate()
                .filter(|(_i, p)| p.exists())
                .filter_map(|(i, p)| image::open(p).ok().map(|img| (i, img)))
                .unzip();

            let results = loaded_model.embed_images(valid_images)?;

            let mut mapped_results = vec![None; image_batch.len()];

            // map them back to a new array so it keeps the original indices and invalid images are None
            for (orig_idx, result) in indices_of_original_array.into_iter().zip(results) {
                mapped_results[orig_idx] = Some(result);
            }

            all_results.extend(mapped_results);
        }

        Ok(all_results)
    }

    /// loads the model to the memory so the images don't have to wait for the model to load
    /// on every batch
    pub fn load_image_embedding_model_to_memory(&mut self, name: &str) -> Result<()> {
        let model = self
            .models
            .get_downloaded_image_embedding_model(name)
            .context(format!("model with name {} not found", name))?;

        let model_bytes = fs::read(model.paths.model)?;
        let preprocessor_config_bytes = fs::read(model.paths.preprocessor_config)?;

        let loaded = ImageEmbedding::try_new_from_user_defined(
            UserDefinedImageEmbeddingModel::new(model_bytes, preprocessor_config_bytes),
            ImageInitOptionsUserDefined::new(),
        )?;

        self.loaded_image_embedding_model
            .insert(name.to_owned(), loaded);

        Ok(())
    }

    /// drops a single loaded image embedding model from the memory
    pub fn drop_loaded_image_embedding_model(&mut self, name: &str) {
        self.loaded_image_embedding_model.remove(name);
    }

    /// drops every loaded image embedding model from the memory
    pub fn drop_loaded_image_embedding_models(&mut self) {
        self.loaded_image_embedding_model.clear();
    }
}

impl ModelManagerModels {
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
}
