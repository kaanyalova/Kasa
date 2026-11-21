use std::path::Path;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{Context, Result, anyhow, bail};
use kasa_ai::model_manager::{AiProgress, ModelManager};
use kasa_ai::model_sources::ModelDownloadProgress;
use log::{debug, error, info};
use sqlx::{Pool, Sqlite};
use tokio::sync::mpsc;

use crate::config::global_config::{GlobalConfig, get_config_impl};
use crate::db::schema::OcrData;
use crate::downloaders::download_queue::DownloadJob;

pub mod model_download_queue;
pub mod ocr_types;

/// How many jobs can sit in the queue before [AiWorker::submit] waits for room
const JOB_QUEUE_SIZE: usize = 32;

struct AiWorker {
    model_manager: Option<Arc<ModelManager>>,
    db: Option<Pool<Sqlite>>,
    thumbs_db: Option<Pool<Sqlite>>,
    config: Option<GlobalConfig>,

    jobs_tx: mpsc::Sender<AIJob>,
}

pub fn job_channel() -> (mpsc::Sender<AIJob>, mpsc::Receiver<AIJob>) {
    mpsc::channel(JOB_QUEUE_SIZE)
}

pub enum AIJob {
    Download(DownloadJob),
    OcrImageBatch(Vec<String>),
    ImageEmbeddingBatch(Vec<String>),
    TextEmbedding(String),
    WdvTaggingBatch(Vec<String>),
}

impl AIJob {
    /// `DownloadJob` isn't `Debug`, so jobs are identified by kind in logs
    fn kind(&self) -> &'static str {
        match self {
            Self::Download(_) => "download",
            Self::OcrImageBatch(_) => "ocr_image_batch",
            Self::ImageEmbeddingBatch(_) => "image_embedding_batch",
            Self::TextEmbedding(_) => "text_embedding",
            Self::WdvTaggingBatch(_) => "wdv_tagging_batch",
        }
    }
}

#[derive(Clone, Debug)]
pub enum AiStateUpdate {
    OnDone,
    OnError { error: String },
}

impl AiWorker {
    pub fn uninitialized(jobs_tx: mpsc::Sender<AIJob>) -> Self {
        Self {
            model_manager: None,
            db: None,
            thumbs_db: None,
            config: None,
            jobs_tx,
        }
    }

    /// A clone of the queue's sending end, for callers that only need to enqueue
    pub fn job_sender(&self) -> mpsc::Sender<AIJob> {
        self.jobs_tx.clone()
    }

    /// Queue a job, waiting for room if the queue is full
    pub async fn submit(&self, job: AIJob) -> Result<()> {
        self.jobs_tx
            .send(job)
            .await
            .map_err(|_| anyhow!("the ai worker job loop is not running"))
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
        let onnx_rt_path = PathBuf::from(&config.ai.onnx_rt_path);
        let models_path = PathBuf::from(&config.ai.models_base_path);

        self.model_manager = Some(Arc::new(ModelManager::new(
            &onnx_rt_path,
            &models_path,
            Box::new(on_download_progress),
            Box::new(on_ai_progress),
        )));

        self.db = Some(db);
        self.thumbs_db = Some(thumbs_db);
        self.config = Some(config);
    }

    pub async fn run<F>(
        &mut self,
        mut jobs_rx: mpsc::Receiver<AIJob>,
        on_update: impl Fn(AiStateUpdate) + Send + Sync + Clone + 'static,
    ) {
        info!("ai worker waiting for jobs...");

        while let Some(job) = jobs_rx.recv().await {
            let kind = job.kind();
            debug!("ai worker picked up a {kind} job");

            if self.db.is_none() || self.thumbs_db.is_none() || self.config.is_none() {
                error!("ai worker is not initialized, dropping the {kind} job");
                continue;
            }

            if let Err(e) = process_ai_job(self, &on_update, job).await {
                error!("failed to process the {kind} job: {e}");
            }
        }

        info!("ai worker job loop stopped, every sender was dropped");
    }
}

/// Dispatches a single job to its handler and reports the outcome to `on_update`
async fn process_ai_job(
    ai_worker: &mut AiWorker,
    on_update: &(impl Fn(AiStateUpdate) + Send + Sync + Clone + 'static),
    job: AIJob,
) -> Result<()> {
    let config = get_config_impl();

    let db = ai_worker.db.clone().unwrap();
    let thumbs_db = ai_worker.thumbs_db.clone().unwrap();

    let result = match job {
        AIJob::Download(job) => {
            process_download_job(ai_worker, &config, &db, &thumbs_db, job).await
        }
        AIJob::OcrImageBatch(images) => process_ocr_job(ai_worker, &config, images).await,
        AIJob::ImageEmbeddingBatch(images) => {
            process_image_embedding_job(ai_worker, &config, &db, &thumbs_db, images).await
        }
        AIJob::TextEmbedding(text) => {
            process_text_embedding_job(ai_worker, &config, &db, &thumbs_db, text).await
        }
        AIJob::WdvTaggingBatch(images) => {
            process_wdv_tagging_job(ai_worker, &config, &db, &thumbs_db, images).await
        }
    };

    match &result {
        Ok(()) => on_update(AiStateUpdate::OnDone),
        Err(e) => on_update(AiStateUpdate::OnError {
            error: e.to_string(),
        }),
    }

    result
}

async fn process_ocr_job(
    ai_worker: &mut AiWorker,
    config: &GlobalConfig,
    items: Vec<String>,
) -> Result<()> {
    let model_name = config.ai.ocr_model.clone();

    let manager = ai_worker
        .model_manager
        .clone()
        .context("ai worker is not initialized")?;

    if manager
        .models
        .get_downloaded_ocr_model(&model_name)
        .is_none()
    {
        return Err(anyhow!("model {} is not downloaded", model_name));
    }

    //let regions =
    //    tokio::task::spawn_blocking(move || manager.ocr_images(&model_name, &items)).await??;
    //
    //let ocred_count = regions.iter().filter(|r| r.is_some()).count();
    //
    //info!("ocred {ocred_count}/{} images", regions.len());
    //

    Ok(())
}

#[allow(unused_variables)]
async fn process_download_job(
    ai_worker: &mut AiWorker,
    config: &GlobalConfig,
    db: &Pool<Sqlite>,
    thumbs_db: &Pool<Sqlite>,
    job: DownloadJob,
) -> Result<()> {
    // downloads are already handled by the downloader queue, the ai worker only
    // exists to run the models
    bail!("download jobs are not handled by the ai worker")
}

#[allow(unused_variables)]
async fn process_image_embedding_job(
    ai_worker: &mut AiWorker,
    config: &GlobalConfig,
    db: &Pool<Sqlite>,
    thumbs_db: &Pool<Sqlite>,
    items: Vec<String>,
) -> Result<()> {
    // TODO: calculate_embeddings_for_images + insert_embeddings
    bail!("image embedding jobs are not implemented yet")
}

#[allow(unused_variables)]
async fn process_text_embedding_job(
    ai_worker: &mut AiWorker,
    config: &GlobalConfig,
    db: &Pool<Sqlite>,
    thumbs_db: &Pool<Sqlite>,
    text: String,
) -> Result<()> {
    // TODO: calculate_embeddings_for_text, the embedding models are kept loaded
    // in the model manager so this should never need to load a model
    bail!("text embedding jobs are not implemented yet")
}

#[allow(unused_variables)]
async fn process_wdv_tagging_job(
    ai_worker: &mut AiWorker,
    config: &GlobalConfig,
    db: &Pool<Sqlite>,
    thumbs_db: &Pool<Sqlite>,
    items: Vec<String>,
) -> Result<()> {
    // TODO: tag_images_wdv + insert_tags_with_source_types
    bail!("wdv tagging jobs are not implemented yet")
}
