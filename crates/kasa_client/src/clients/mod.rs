mod database;
mod downloader;
mod local;
mod remote;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use kasa_core::{
    db::{
        TagQueryOutput,
        embeddings::EmbeddingDistance,
        schema::{Media, MediaSource},
    },
    downloaders::download_queue::DownloaderStateUpdate,
    media::{MediaInfo, SourceCategoryGroupedTags, TagWithDetails},
    tags::{AllTagsOrderingCriteria, TagWithCount, search::SearchCriteria},
};
use tokio::sync::mpsc;

use crate::events::KasaEvent;

type EventCallback = Arc<dyn Fn(KasaEvent) + Send + Sync + 'static>;

#[async_trait]
pub trait KasaClient: Send + Sync {
    fn call_event(&self, event: KasaEvent);

    fn set_event_handler(&mut self, callback: EventCallback);

    async fn get_info(&self, hash: &str) -> Result<Option<MediaInfo>>;

    async fn get_tags(&self, hash: &str) -> Result<Vec<TagWithDetails>>;

    async fn get_media_type(&self, hash: &str) -> Result<String>;

    async fn get_tags_grouped_by_source_categories(
        &self,
        hash: &str,
    ) -> Result<SourceCategoryGroupedTags>;

    async fn get_media_name(&self, hash: &str) -> Result<String>;

    async fn get_media_sources(&self, hash: &str) -> Result<Vec<MediaSource>>;

    async fn set_media_favorite(&self, hash: &str, is_favorite: bool) -> Result<()>;

    async fn get_video_length(&self, hash: &str) -> Result<Option<f64>>;

    async fn get_top_n_closest_for_media(
        &self,
        hash: &str,
        n: i64,
    ) -> Result<Vec<EmbeddingDistance>>;

    async fn get_valid_path(&self, hash: &str) -> Result<String>;

    async fn query_tags(&self, query: &str, limit: i64) -> Result<Vec<TagQueryOutput>>;

    async fn get_thumbnail(&self, hash: &str) -> Result<Vec<u8>>;

    async fn serve_media(&self, hash: &str) -> Result<Vec<u8>>;

    async fn search(&self, criteria: SearchCriteria) -> Result<Vec<Media>>;

    async fn update_tags(&self, raw_input: &str, hash: &str) -> Result<()>;

    async fn delete_tags(&self, hash: &str, tags: &[String]) -> Result<()>;

    async fn get_tags_as_text(&self, hash: &str) -> Result<Option<String>>;

    async fn get_list_of_all_tags_with_details(
        &self,
        ordering_criteria: AllTagsOrderingCriteria,
    ) -> Result<Vec<TagWithCount>>;

    async fn push_download(&self, url: &str) -> Result<()>;
}
