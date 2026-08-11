use std::{
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender},
    },
    thread::JoinHandle,
};

use anyhow::Result;

use crate::{GalleryDlOutput, GalleryDlStatus, gdl_download, init_interpreter_with_gallery_dl};

pub struct DownloadJob {
    pub url: String,
}

struct GalleryDlWorkerDownloadJob {
    url: String,
    output_path: String,
    gdl_config_path: Option<String>,
    when_done: Sender<Result<GalleryDlOutput>>,
}

pub struct GalleryDlDownloadWorker {
    thread_handle: Option<JoinHandle<()>>,
    job_tx: Option<Sender<GalleryDlWorkerDownloadJob>>,
}

impl Drop for GalleryDlDownloadWorker {
    fn drop(&mut self) {
        self.job_tx.take();
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl GalleryDlDownloadWorker {
    pub fn new(
        on_progress: impl Fn(&GalleryDlStatus) + Send + Sync + Clone + 'static,
    ) -> Result<Self> {
        let (job_tx, job_rx): (
            Sender<GalleryDlWorkerDownloadJob>,
            Receiver<GalleryDlWorkerDownloadJob>,
        ) = mpsc::channel();

        let on_progress = Arc::new(on_progress) as Arc<dyn Fn(&GalleryDlStatus) + Send + Sync>;
        let on_progress_cloned = on_progress.clone();

        let thread = std::thread::Builder::new()
            .name("rustpython_gallery_dl_worker".to_owned())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let interpreter = init_interpreter_with_gallery_dl();

                while let Ok(job) = job_rx.recv() {
                    let on_progress = on_progress_cloned.clone();
                    let on_progress_callback = move |status: &GalleryDlStatus| on_progress(status);

                    let result = gdl_download(
                        &interpreter,
                        &job.url,
                        &job.output_path,
                        job.gdl_config_path.as_deref(),
                        on_progress_callback,
                    );

                    let _ = job.when_done.send(result);
                }
            })?;

        Ok(Self {
            thread_handle: Some(thread),
            job_tx: Some(job_tx),
        })
    }

    pub fn push_job(
        &self,
        job: DownloadJob,
        output_path: &str,
        gdl_config_path: Option<&str>,
    ) -> Result<Receiver<Result<GalleryDlOutput>>> {
        let (done_tx, done_rx) = mpsc::channel();

        if let Some(tx) = &self.job_tx {
            let job = GalleryDlWorkerDownloadJob {
                url: job.url,
                output_path: output_path.to_string(),
                gdl_config_path: gdl_config_path.map(|s| s.to_string()),
                when_done: done_tx,
            };
            tx.send(job)?;

            Ok(done_rx)
        } else {
            Err(anyhow::anyhow!(
                "trying to push a job to a worker thread that already shut down"
            ))
        }
    }
}
