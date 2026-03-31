use core::error;
use std::{
    collections::HashMap,
    fs::{self, FileType},
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{
    ExtractedTag, PyTrustMe,
    extractors::TagExtractor,
    extractors::{ExtractedTags, NoTagsInfo},
};
use anyhow::{Ok, Result};
use rustpython::Interpreter;
use rustpython_vm::{compiler, py_compile, vm};
use serde::{Deserialize, Serialize};
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
                Some((contents, file_name))
            })
            .collect())
    }

    fn extractors(&self) -> &HashMap<String, String>;

    fn get_extractor_code(&self, name: &str) -> Option<&String> {
        self.extractors().get(name)
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
    // can i get rid of the arc mutex here?
    interpreter: Arc<Mutex<PyTrustMe>>,
    extractors: HashMap<String, String>,
}

impl PythonTagExtractor {
    pub fn init(interpreter: Arc<Mutex<PyTrustMe>>, base_dir: &Path) -> Result<Self> {
        let extractors = Self::parse_extractor_files(base_dir)?;

        Ok(Self {
            interpreter: interpreter.clone(),
            extractors,
        })
    }
}

impl TagExtractor for PythonTagExtractor {
    fn extract_tags(&self, extractor: &str, json_input: &Value) -> Result<ExtractedTags> {
        let result = self.execute_extractor(extractor, json_input)?;
        Ok(result.unwrap_or(ExtractedTags::NoTags(NoTagsInfo::NoExtractorFound)))
    }
}

impl ScriptableTagExtractor for PythonTagExtractor {
    fn execute_extractor(
        &self,
        extractor: &str,
        json_input: &Value,
    ) -> Result<Option<ExtractedTags>> {
        let code = self.get_extractor_code(extractor);
        if code.is_none() {
            return Ok(None);
        }

        let code = code.unwrap().as_str();

        let output_string: Result<String> = self.interpreter.lock().unwrap().0.enter(|vm| {
            let scope = vm.new_scope_with_builtins();

            let compiled = vm
                .compile(code, compiler::Mode::Exec, "<embedded>".to_string())
                .map_err(|err| {
                    ScriptableTagExtractorError::PythonError(format!("Compile error: {:?}", err))
                })?;

            vm.run_code_obj(compiled, scope.clone()).map_err(|err| {
                ScriptableTagExtractorError::PythonError(format!("Runtime error: {:?}", err))
            })?;

            let parser_function = scope.globals.get_item("parse", vm).map_err(|_| {
                ScriptableTagExtractorError::PythonError("Function 'parse' not found".to_string())
            })?;

            let json = serde_json::to_string(json_input).unwrap();
            let input = vm.ctx.new_str(json);

            let result = parser_function.call((input,), vm).map_err(|err| {
                ScriptableTagExtractorError::PythonError(format!("Execution error: {:?}", err))
            })?;

            let json_module = vm.import("json", 0).map_err(|err| {
                ScriptableTagExtractorError::PythonError(format!(
                    "Failed to import json module: {:?}",
                    err
                ))
            })?;

            let dumps_function = json_module.get_attr("dumps", vm).map_err(|err| {
                ScriptableTagExtractorError::PythonError(format!("json.dumps not found: {:?}", err))
            })?;

            let json_result = dumps_function.call((result,), vm).map_err(|err| {
                ScriptableTagExtractorError::PythonError(format!("Serialization error: {:?}", err))
            })?;

            let output = json_result
                .str(vm)
                .map_err(|_| {
                    ScriptableTagExtractorError::PythonError(
                        "Output string conversion error".to_string(),
                    )
                })?
                .as_str()
                .to_string();

            Ok(output)
        });

        let json_result_string = output_string?;

        let serialized: ExtractedTags = serde_json::from_str(&json_result_string)?;

        Ok(Some(serialized))
    }

    fn extractors(&self) -> &HashMap<String, String> {
        &self.extractors
    }
}
