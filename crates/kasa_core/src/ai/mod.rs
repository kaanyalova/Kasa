use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use kasa_ai::model_manager::{AiProgress, ModelManager};
use kasa_ai::model_sources::ModelDownloadProgress;
use log::{debug, error, info};
use sqlx::{Pool, Sqlite};
use tokio::sync::mpsc;

use crate::config::global_config::{GlobalConfig, get_config_impl};
use crate::downloaders::download_queue::DownloadJob;

pub mod ocr_types;

/// How many jobs can sit in the queue before [AiWorker::submit] waits for room
const JOB_QUEUE_SIZE: usize = 32;

struct AiWorker {
    model_manager: Option<ModelManager>,
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
    OcrImageBatch(Vec<PathBuf>),
    ImageEmbeddingBatch(Vec<PathBuf>),
    TextEmbedding(String),
    WdvTaggingBatch(Vec<PathBuf>),
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

        self.model_manager = Some(ModelManager::new(
            &onnx_rt_path,
            &models_path,
            Box::new(on_download_progress),
            Box::new(on_ai_progress),
        ));

        self.db = Some(db);
        self.thumbs_db = Some(thumbs_db);
        self.config = Some(config);
    }

    pub async fn run<F>(&mut self, mut jobs_rx: mpsc::Receiver<AIJob>, on_update: F)
    where
        F: Fn(AiStateUpdate) + Send + Sync + Clone + 'static,
    {
        info!("ai worker waiting for jobs...");

        while let Some(job) = jobs_rx.recv().await {
            let kind = job.kind();
            debug!("ai worker picked up a {kind} job");

            let (Some(db), Some(thumbs_db), Some(config)) =
                (self.db.clone(), self.thumbs_db.clone(), self.config.clone())
            else {
                error!("ai worker is not initialized, dropping the {kind} job");
                continue;
            };

            if let Err(e) = process_ai_job(self, &db, &thumbs_db, &config, &on_update, job).await {
                error!("failed to process the {kind} job: {e}");
            }
        }

        info!("ai worker job loop stopped, every sender was dropped");
    }
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
    match job {
        AIJob::OcrImageBatch(items) => {
            let model_name = config.ai.ocr_model.clone();
            let paths: Vec<PathBuf> = items.into_iter().map(PathBuf::from).collect();

            let manager = ai_worker
                .model_manager
                .as_mut()
                .context("ai worker is not initialized")?;

            // the pipeline can only be built once the model files are on disk
            if manager
                .models
                .get_downloaded_ocr_model(&model_name)
                .is_none()
            {
                todo!()
            }

            // ocr_images is a blocking cpu call, so it belongs on the blocking pool.
            // that means handing the manager over and taking it back afterwards
            let mut manager = ai_worker
                .model_manager
                .take()
                .context("ai worker is not initialized")?;

            let (manager, regions) = tokio::task::spawn_blocking(move || {
                let regions = manager.ocr_images(&model_name, &paths);
                (manager, regions)
            })
            .await?;

            ai_worker.model_manager = Some(manager);

            let regions = regions?;
            let ocred_count = regions.iter().filter(|r| r.is_some()).count();

            // TODO: write the regions to OcrData, keyed by the hash of each path
            info!("ocred {ocred_count}/{} images", regions.len());

            on_update(AiStateUpdate::OnDone);
        }

        // TODO: implement the remaining job kinds
        AIJob::Download(_)
        | AIJob::ImageEmbeddingBatch(_)
        | AIJob::TextEmbedding(_)
        | AIJob::WdvTaggingBatch(_) => {}
    }

    Ok(())
}
