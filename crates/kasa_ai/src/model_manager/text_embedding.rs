use std::fs;

use anyhow::{Context, Result};
use fastembed::{InitOptionsUserDefined, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel};

use super::{ModelManager, ModelManagerModels};
use crate::model_sources::{Model, TextEmbeddingModel};

impl ModelManager {
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

    /// loads the model to the memory so the user can make searches without waiting for the model
    /// to load every time
    pub fn load_text_embedding_model_to_memory(&mut self, name: &str) -> Result<()> {
        let model = self
            .models
            .get_downloaded_text_embedding_model(name)
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

    /// drops a single loaded text embedding model from the memory
    pub fn drop_loaded_embedding_model(&mut self, name: &str) {
        self.loaded_text_embedding_model.remove(name);
    }

    /// drops every loaded text embedding model from the memory
    pub fn drop_loaded_embedding_models(&mut self) {
        self.loaded_text_embedding_model.clear();
    }
}

impl ModelManagerModels {
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
