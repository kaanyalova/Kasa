use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use kasa_ai::model_manager::{AiProgress, ModelManager};
use kasa_ai::model_sources::ModelDownloadProgress;
use sqlx::{Pool, Sqlite};

use crate::config::global_config::GlobalConfig;

pub mod ocr_types;

struct AiWorker {
    model_manager: Option<ModelManager>,
}

pub enum AIJob {
    Download(DownloadJob),
    OcrImageBatch(Vec<String>),
    ImageEmbeddingBatch(Vec<String>),
    TextEmbedding(String),
    WdvTaggingBatch(Vec<String>),
}

#[derive(Clone, Debug)]
pub enum AiStateUpdate {
    OnDone,
    OnError { error: String },
}

impl AiWorker {
    pub fn uninitialized() -> Self {
        Self {
            model_manager: None,
        }
    }

    pub fn init<FDP, FAP>(
        &mut self,
        config: GlobalConfig,
        db: Pool<Sqlite>,
        thumbs_db: Pool<Sqlite>,
        on_download_progress: FDP,
        on_ai_progress: FAP,
    ) where
        FDP: Fn(ModelDownloadProgress) + Send + Sync + 'static,
        FAP: Fn(AiProgress) + Send + Sync + 'static,
    {
        let onnx_rt_path = PathBuf::from(config.ai.onnx_rt_path);
        let models_path = PathBuf::from(config.ai.models_base_path);

        self.model_manager = Some(ModelManager::new(
            &onnx_rt_path,
            &models_path,
            Box::new(on_download_progress),
            Box::new(on_ai_progress),
        ));

        // self.model_manager =
        //    ModelManager::new(rt_path, models_path, on_download_progress, on_ai_progress)
    }

    pub fn run() {}
}

#[allow(dead_code, unused_variables)]
async fn process_ai_job<F>(
    ai_worker: &mut AiWorker,
    db: &Pool<Sqlite>,
    thumbs_db: &Pool<Sqlite>,
    config: &GlobalConfig,
    on_update: &F,
    job: AIJob,
) -> Result<()>
where
    F: Fn(AiStateUpdate) + Send + Sync + Clone + 'static,
{
    todo!()
}
