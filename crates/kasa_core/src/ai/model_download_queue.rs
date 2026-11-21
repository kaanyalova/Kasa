use anyhow::{Result, anyhow};
use kasa_ai::model_manager::ModelManager;
use log::{debug, error, info};
use tokio::sync::mpsc;

const JOB_QUEUE_SIZE: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelKind {
    Ocr,
    Wdv,
    ImageEmbedding,
    TextEmbedding,
}

impl ModelKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ocr => "ocr",
            Self::Wdv => "wdv",
            Self::ImageEmbedding => "image_embedding",
            Self::TextEmbedding => "text_embedding",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelDownloadJob {
    pub kind: ModelKind,
    pub model_name: String,
}

impl ModelDownloadJob {
    pub fn new(kind: ModelKind, model_name: impl Into<String>) -> Self {
        Self {
            kind,
            model_name: model_name.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ModelDownloadStateUpdate {
    OnDone {
        kind: ModelKind,
        model_name: String,
    },
    OnError {
        kind: ModelKind,
        model_name: String,
        error: String,
    },
}

pub fn job_channel() -> (
    mpsc::Sender<ModelDownloadJob>,
    mpsc::Receiver<ModelDownloadJob>,
) {
    mpsc::channel(JOB_QUEUE_SIZE)
}

pub async fn submit(jobs_tx: &mpsc::Sender<ModelDownloadJob>, job: ModelDownloadJob) -> Result<()> {
    jobs_tx
        .send(job)
        .await
        .map_err(|_| anyhow!("the model download queue is not running"))
}

pub async fn run(
    manager: std::sync::Arc<ModelManager>,
    mut jobs_rx: mpsc::Receiver<ModelDownloadJob>,
    on_update: impl Fn(ModelDownloadStateUpdate) + Send + Sync + Clone + 'static,
) {
    info!("model download worker waiting for jobs...");

    while let Some(job) = jobs_rx.recv().await {
        debug!(
            "downloading the {} model {}",
            job.kind.as_str(),
            job.model_name
        );

        let result = match job.kind {
            ModelKind::Ocr => manager.models.download_ocr_model(&job.model_name).await,
            ModelKind::Wdv => manager.models.download_wdv_model(&job.model_name).await,
            ModelKind::ImageEmbedding => {
                manager
                    .models
                    .download_image_embedding_model(&job.model_name)
                    .await
            }
            ModelKind::TextEmbedding => {
                manager
                    .models
                    .download_text_embedding_model(&job.model_name)
                    .await
            }
        };

        match result {
            Ok(()) => {
                info!(
                    "downloaded the {} model {}",
                    job.kind.as_str(),
                    job.model_name
                );

                on_update(ModelDownloadStateUpdate::OnDone {
                    kind: job.kind,
                    model_name: job.model_name,
                });
            }
            Err(e) => {
                error!(
                    "failed to download the {} model {}: {e}",
                    job.kind.as_str(),
                    job.model_name
                );

                on_update(ModelDownloadStateUpdate::OnError {
                    kind: job.kind,
                    model_name: job.model_name,
                    error: e.to_string(),
                });
            }
        }
    }

    info!("model download worker stopped, every sender was dropped");
}
