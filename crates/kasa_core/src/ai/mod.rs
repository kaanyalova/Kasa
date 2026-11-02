use std::collections::HashMap;

use kasa_ai::model_manager::{AiProgress, ModelManager};

pub mod ocr_types;

struct AiWorker {
    model_manager: Option<ModelManager>,
}

struct AiWorkerContext {
    ai_progress: HashMap<u64, AiProgress>,
}

enum AIJob {
    OcrImageBatch(Vec<String>),
    ImageEmbeddingBatch(Vec<String>),
    TextEmbedding(String),
    WdvTaggingBatch(Vec<String>),
}

impl AiWorker {
    pub fn uninitialized() -> Self {
        Self {
            model_manager: None,
        }
    }

    pub fn initialize(&mut self) {
        // self.model_manager =
        //    ModelManager::new(rt_path, models_path, on_download_progress, on_ai_progress)
    }

    fn update_progress_state(&mut self, progress: AiProgress) {
        self.ai_progress.insert(progress.inner().id, progress);
    }
}
