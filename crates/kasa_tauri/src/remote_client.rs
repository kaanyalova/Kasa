use std::env;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use kasa_core::{
    db::{
        TagQueryOutput,
        embeddings::EmbeddingDistance,
        schema::{Media, MediaSource},
    },
    downloaders::download_queue::{DownloadJob, DownloaderStateUpdate},
    media::{MediaInfo, SourceCategoryGroupedTags, TagWithDetails},
    tags::{AllTagsOrderingCriteria, TagWithCount, search::SearchCriteria},
};
use log::{trace, warn};
use serde_json::json;
use tokio::time::{Duration, timeout};
use tokio_tungstenite::connect_async;
use tokio_util::sync::CancellationToken;

#[derive(Default, Clone)]
pub struct RemoteClient {
    reqwest_client: reqwest::Client,
    base_url: String,
}

// my plan was to generate these using openapi, but theres no rust crate that can do it from
// openapi 3.1, so i am stuck with manually implementing these
impl RemoteClient {
    pub fn new(base_url: &str) -> Self {
        // brotli should be enabled by default if i enable the feature but just in case
        let mut reqwest_client = reqwest::Client::builder().brotli(true);

        if env::var("KASA_REMOTE_CLIENT_VERBOSE") == Ok("1".to_string()) {
            trace!("KASA_REMOTE_CLIENT_VERBOSE is on, enabling reqwest verbose logging");
            reqwest_client = reqwest_client.connection_verbose(true);
        }

        Self {
            reqwest_client: reqwest_client.build().unwrap(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn ping(&self) -> Result<String> {
        let url = format!("{}/ping", self.base_url);
        let response = self.reqwest_client.get(&url).send().await?;
        Ok(response.text().await?)
    }

    pub async fn get_thumbnail(&self, hash: &str) -> Result<Vec<u8>> {
        let url = format!("{}/get_thumbnail", self.base_url);
        let response = self
            .reqwest_client
            .get(&url)
            .query(&[("hash", hash)])
            .send()
            .await?;
        Ok(response.bytes().await?.to_vec())
    }

    pub async fn query_tags(&self, query: &str, limit: i64) -> Result<Vec<TagQueryOutput>> {
        let url = format!("{}/query_tags", self.base_url);
        let response: Vec<TagQueryOutput> = self
            .reqwest_client
            .get(&url)
            .query(&[("query", query), ("limit", &limit.to_string())])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn get_info(&self, hash: &str) -> Result<Option<MediaInfo>> {
        let url = format!("{}/get_info", self.base_url);
        let response: Option<MediaInfo> = self
            .reqwest_client
            .get(&url)
            .query(&[("hash", hash)])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn get_tags(&self, hash: &str) -> Result<Vec<TagWithDetails>> {
        let url = format!("{}/get_tags", self.base_url);
        let response: Vec<TagWithDetails> = self
            .reqwest_client
            .get(&url)
            .query(&[("hash", hash)])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn get_media_type(&self, hash: &str) -> Result<String> {
        let url = format!("{}/get_media_type", self.base_url);
        let response: String = self
            .reqwest_client
            .get(&url)
            .query(&[("hash", hash)])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn get_tags_grouped_by_source_categories(
        &self,
        hash: &str,
    ) -> Result<SourceCategoryGroupedTags> {
        let url = format!("{}/get_tags_grouped_by_source_categories", self.base_url);
        let response: SourceCategoryGroupedTags = self
            .reqwest_client
            .get(&url)
            .query(&[("hash", hash)])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn get_media_sources(&self, hash: &str) -> Result<Vec<MediaSource>> {
        let url = format!("{}/get_media_sources", self.base_url);
        let response: Vec<MediaSource> = self
            .reqwest_client
            .get(&url)
            .query(&[("hash", hash)])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn set_media_favorite(&self, hash: &str, is_favorite: bool) -> Result<()> {
        let url = format!("{}/set_media_favorite", self.base_url);
        self.reqwest_client
            .put(&url)
            .query(&[("hash", hash), ("is_favorite", &is_favorite.to_string())])
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_video_length(&self, hash: &str) -> Result<Option<f64>> {
        let url = format!("{}/get_video_length", self.base_url);
        let response: Option<f64> = self
            .reqwest_client
            .get(&url)
            .query(&[("hash", hash)])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn get_top_n_closest_for_media(
        &self,
        hash: &str,
        n: i64,
    ) -> Result<Vec<EmbeddingDistance>> {
        let url = format!("{}/get_top_n_closest_for_media", self.base_url);
        let response: Vec<EmbeddingDistance> = self
            .reqwest_client
            .get(&url)
            .query(&[("hash", hash), ("n", &n.to_string())])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn search(&self, query: &SearchCriteria) -> Result<Vec<Media>> {
        let url = format!("{}/search", self.base_url);
        let response: Vec<Media> = self
            .reqwest_client
            .post(&url)
            .json(query)
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn update_tags(&self, raw_input: &str, hash: &str) -> Result<()> {
        let url = format!("{}/update_tags", self.base_url);
        self.reqwest_client
            .put(&url)
            .query(&[("raw_input", raw_input), ("hash", hash)])
            .send()
            .await?;

        Ok(())
    }

    pub async fn delete_tags(&self, hash: &str, tags: Vec<String>) -> Result<()> {
        let url = format!("{}/delete_tags", self.base_url);
        self.reqwest_client
            .delete(&url)
            .query(&json!({ "hash": hash, "tags": tags }))
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_tags_as_text(&self, hash: &str) -> Result<Option<String>> {
        let url = format!("{}/get_tags_as_text", self.base_url);
        let response: Option<String> = self
            .reqwest_client
            .get(&url)
            .query(&[("hash", hash)])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn get_list_of_all_tags_with_details(
        &self,
        ordering_criteria: AllTagsOrderingCriteria,
    ) -> Result<Vec<TagWithCount>> {
        let url = format!("{}/get_list_of_all_tags_with_details", self.base_url);
        let response: Vec<TagWithCount> = self
            .reqwest_client
            .get(&url)
            .query(&[("ordering_criteria", &ordering_criteria.to_string())])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn get_media_name(&self, hash: &str) -> Result<String> {
        let url = format!("{}/get_media_name", self.base_url);
        let response: String = self
            .reqwest_client
            .get(&url)
            .query(&[("hash", hash)])
            .send()
            .await?
            .text()
            .await?;

        Ok(response)
    }

    pub async fn get_valid_path(&self, hash: &str) -> Result<String> {
        let url = format!("{}/get_valid_path", self.base_url);
        let response: String = self
            .reqwest_client
            .get(&url)
            .query(&[("hash", hash)])
            .send()
            .await?
            .text()
            .await?;
        Ok(response)
    }

    pub fn url(&self) -> String {
        self.base_url.clone()
    }
}

pub struct RemoteDownloaderClient {
    reqwest_client: reqwest::Client,
    base_url: String,
    cancel_token: Option<CancellationToken>,
}

impl RemoteDownloaderClient {
    pub fn new(base_url: &str) -> Self {
        let reqwest_client = reqwest::Client::builder().brotli(true).build().unwrap();

        Self {
            reqwest_client,
            base_url: base_url.to_string(),
            cancel_token: None,
        }
    }

    pub async fn push_download(&self, download_url: &str) -> Result<()> {
        let url = format!("{}/push_download", self.base_url);
        self.reqwest_client
            .post(&url)
            .json(&DownloadJob {
                url: download_url.to_string(),
            })
            .send()
            .await?;

        Ok(())
    }

    pub async fn listen_for_downloader_updates(
        &mut self,
        on_update: impl Fn(DownloaderStateUpdate) + Send + Sync + 'static,
    ) -> Result<()> {
        if self.cancel_token.is_some() {
            warn!("trying to listen for downloader updates while already listening");
            return Ok(());
        }

        let token = CancellationToken::new();
        self.cancel_token = Some(token.clone());

        let ws_base_url = self
            .base_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");
        let url = format!("{}/listen_for_download_updates", ws_base_url);

        // connection + upgrade timeout
        let (ws_stream, _) = timeout(Duration::from_secs(10), connect_async(url)).await??;

        let (mut write, mut read) = ws_stream.split();

        tokio::spawn(async move {
            tokio::select! {
                _ = async {
                    loop {
                        let msg = match timeout(Duration::from_secs(30), read.next()).await {
                            Ok(Some(Ok(msg))) => msg,
                            Ok(Some(Err(_))) | Ok(None) => break,
                            Err(_elapsed) => {
                                warn!(
                                    "30 seconds elapsed since last heartbeat, breaking the ws connection"
                                );
                                break;
                            }
                        };

                        if let Ok(text) = msg.into_text() {
                            if text == r#"{"type":"heartbeat"}"# {
                                continue;
                            }

                            if let Ok(update) = serde_json::from_str::<DownloaderStateUpdate>(&text) {
                                on_update(update);
                            }
                        }
                    }
                } => {},
                _ = token.cancelled() => {
                    let _ = write.close().await;
                }
            }
        });

        Ok(())
    }
}

impl Drop for RemoteDownloaderClient {
    fn drop(&mut self) {
        if let Some(token) = self.cancel_token.take() {
            token.cancel();
        }
    }
}
