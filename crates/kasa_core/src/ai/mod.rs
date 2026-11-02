use std::collections::HashMap;

use kasa_ai::model_manager::{AiProgress, ModelManager};

pub mod ocr_types;

struct AIWorker {
    model_manager: Option<ModelManager>,
    ai_progress: HashMap<u64, AiProgress>,
}

enum AIJob {
    OcrImageBatch(Vec<String>),
    ImageEmbeddingBatch(Vec<String>),
    TextEmbedding(String),
    WdvTagingBatch(Vec<String>),
}

impl AIWorker {
    pub fn uninitialized() -> Self {
        Self {
            model_manager: None,
            ai_progress: HashMap::new(),
        }
    }

    pub fn initialize(&mut self) {
        self.model_manager =
            ModelManager::new(rt_path, models_path, on_download_progress, on_ai_progress)
    }

    fn update_progress_state(&mut self, progress: AiProgress) {
        self.ai_progress.insert(progress.inner().id, progress);
    }
}
