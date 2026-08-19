use std::{
    collections::HashMap,
    fs::{self},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::{
    extractors::{ExtractedTags, NoTagsInfo, TagExtractor},
    worker::tagger::TaggerWorker,
};
use anyhow::{Ok, Result};
use rustpython_vm::compiler;
use serde_json::Value;
use thiserror::Error;

pub trait ScriptableTagExtractor: TagExtractor {
    fn execute_extractor(
        &self,
        extractor: &str,
        json_input: &Value,
    ) -> Result<Option<ExtractedTags>>;

    fn parse_extractor_files(base_dir: &Path) -> Result<HashMap<String, String>> {
        if !base_dir.is_dir() {
            return Err(ScriptableTagExtractorError::ExtractorPathNotAValidDirectory.into());
        }

        let files = fs::read_dir(base_dir)?;

        let files = files.filter_map(|f| f.ok()).filter(|f| {
            f.file_type().is_ok_and(|ft| ft.is_file())
                && f.path().extension().is_some_and(|ext| ext == "py")
        });

        Ok(files
            .filter_map(|f| {
                let contents = fs::read_to_string(f.path()).ok()?;
                let file_name = f.file_name().into_string().ok()?;
                // strip the python suffixes here
                let name = file_name.strip_suffix(".py").unwrap_or(&file_name);
                if name.is_empty() {
                    return None;
                }
                Some((name.to_owned(), contents))
            })
            .collect())
    }

    fn extractors(&self) -> &Mutex<HashMap<String, String>>;

    fn get_extractor_code(&self, name: &str) -> Option<String> {
        self.extractors().lock().unwrap().get(name).cloned()
    }
}

#[derive(Debug, Error)]
pub enum ScriptableTagExtractorError {
    #[error("the extractor path is not a valid directory")]
    ExtractorPathNotAValidDirectory,
    #[error("extractor name not found in JSON")]
    ExtractorNameNotFound,
    #[error("python error: {0}")]
    PythonError(String),
}

pub struct PythonTagExtractor {
    worker: Arc<TaggerWorker>,
    extractors: Mutex<HashMap<String, String>>,
    base_dir: PathBuf,
}

impl PythonTagExtractor {
    pub fn init(worker: Arc<TaggerWorker>, base_dir: &Path) -> Result<Self> {
        let extractors = Self::parse_extractor_files(base_dir)?;

        Ok(Self {
            worker,
            extractors: Mutex::new(extractors),
            base_dir: base_dir.to_path_buf(),
        })
    }
}

impl TagExtractor for PythonTagExtractor {
    fn extract_tags(&self, extractor: &str, json_input: &Value) -> Result<ExtractedTags> {
        let result = self.execute_extractor(extractor, json_input)?;
        Ok(result.unwrap_or(ExtractedTags::NoTags(NoTagsInfo::NoExtractorFound)))
    }

    fn reload(&self) -> Result<()> {
        let new = Self::parse_extractor_files(&self.base_dir)?;
        *self.extractors.lock().unwrap() = new;
        Ok(())
    }
}

impl ScriptableTagExtractor for PythonTagExtractor {
    fn execute_extractor(
        &self,
        extractor: &str,
        json_input: &Value,
    ) -> Result<Option<ExtractedTags>> {
        let code = self.get_extractor_code(extractor);
        let Some(code) = code else {
            return Ok(None);
        };

        let result = self.worker.push_job(&code, json_input)?;

        // this waits until the pushed job is done, so its effectively single threaded
        // instead of locking a mutex i do this
        let result = result.recv()??;

        Ok(result)
    }

    fn extractors(&self) -> &Mutex<HashMap<String, String>> {
        &self.extractors
    }
}
