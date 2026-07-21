use std::collections::HashMap;

use anyhow::{Result, anyhow};
use log::trace;
use rustpython::{InterpreterBuilder, InterpreterBuilderExt};
use rustpython_pylib::FROZEN_STDLIB;
use rustpython_vm::{convert::ToPyObject, py_freeze, pymodule};
use serde::{Deserialize, Serialize};

use serde_json::Value;
use sha1::{Digest, Sha1};
use thiserror::Error;

use crate::extractors::{ExtractedTag, TagExtractor};
pub mod extractors;

pub use rustpython::Interpreter;

const CERT_BYTES: &[u8] = include_bytes!("../cacert.pem");

pub fn init_interpreter() -> Interpreter {
    InterpreterBuilder::new()
        .init_stdlib()
        .add_frozen_modules(FROZEN_STDLIB)
        .build()
}

pub fn init_interpreter_with_gallery_dl() -> Interpreter {
    // There is no easy way of setting the cert bytes in requests library of python so we have to write it to a file
    // TODO: Error handling
    let data_dir = dirs::data_dir().unwrap();
    let data_dir = data_dir.join("kasa");

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).unwrap();
    }

    let cert_path = data_dir.join("cacert.pem");

    if !cert_path.exists() {
        std::fs::write(cert_path, CERT_BYTES).unwrap();
    }

    let builder = InterpreterBuilder::new().init_stdlib();
    let rust_side_module_def = rust_side::module_def(&builder.ctx);
    let builder = builder
        .add_frozen_modules(FROZEN_STDLIB)
        .add_native_module(rust_side_module_def)
        .add_frozen_modules(py_freeze!(
            module_name = "gallery_dl",
            dir = "../py/dependencies/gallery-dl/gallery_dl-1.31.10"
        ))
        .add_frozen_modules(py_freeze!(
            module_name = "charset_normalizer",
            dir = "../py/dependencies/charset_normalizer/charset_normalizer-3.4.0"
        ))
        .add_frozen_modules(py_freeze!(
            module_name = "idna",
            dir = "../py/dependencies/idna/idna-3.10"
        ))
        .add_frozen_modules(py_freeze!(
            module_name = "requests",
            dir = "../py/dependencies/requests/requests-2.32.3/src"
        ))
        .add_frozen_modules(py_freeze!(
            module_name = "certifi",
            dir = "../py/dependencies/certifi/"
        ))
        .add_frozen_modules(py_freeze!(
            module_name = "urllib3",
            dir = "../py/dependencies/urllib3/urllib3-2.2.3/src"
        ))
        //.add_frozen_modules(py_freeze!(
        //    module_name = "yt_dlp",
        //    dir = "../py/dependencies/yt-dlp/yt-dlp",
        //))
        .add_frozen_modules(py_freeze!(dir = "../py/py_src"))
        .build();

    builder
}

pub fn gdl_download(
    interpreter: &Interpreter,
    url: &str,
    output_path: &str,
    gdl_config_path: Option<String>,
    on_progress: impl Fn(&GalleryDlStatus) + Send + Sync + 'static,
) -> Result<GalleryDlOutput> {
    interpreter.enter(|vm| {
        let module = vm.import("gdl", 0).map_err(|e| {
            PyError::PythonException(
                e.to_pyobject(vm)
                    .try_into_value::<String>(vm)
                    .unwrap_or("Cannot get python error message!".into()),
            )
        })?;
        let func = module.get_attr("download", vm).map_err(|e| {
            PyError::PythonException(
                e.to_pyobject(vm)
                    .try_into_value::<String>(vm)
                    .unwrap_or("Cannot get python error message!".to_string()),
            )
        })?;

        let on_progress_wrapper =
            move |status_json: String, _vm: &rustpython_vm::VirtualMachine| {
                let parsed = &serde_json::from_str(&status_json);

                if let Ok(parsed) = parsed {
                    on_progress(parsed)
                }
            };

        let on_progress = vm.new_function("on_progress", on_progress_wrapper);

        dbg!(&gdl_config_path);

        let output = func
            .call(
                (
                    url,
                    output_path,
                    gdl_config_path.unwrap_or("".to_string()),
                    on_progress,
                ),
                vm,
            )
            .map_err(|e| {
                vm.print_exception(e)

                //PyError::PythonException(
                //    e.to_pyobject(vm).try_into_value::<String>(vm).unwrap(), //.unwrap_or("Cannot get python error message!".to_string()),
                //)
            })
            .map_err(|e| {
                PyError::PythonException(
                    e.to_pyobject(vm)
                        .try_into_value::<String>(vm)
                        .unwrap_or("Cannot get python error".to_string()), //.unwrap_or("Cannot get python error message!".to_string()),
                )
            })?;

        let output: String = output.try_into_value(vm).map_err(|e| {
            PyError::PythonException(
                e.to_pyobject(vm).try_into_value::<String>(vm).unwrap(), //.unwrap_or("Cannot get python error message!".to_string()),
            )
        })?;

        let gdl_output: GalleryDlOutput = serde_json::from_str(&output)?;

        trace!("Raw gallery_dl output: {:#?}", &gdl_output);

        if std::env::var("KASA_GDL_OUTPUT").is_ok() {
            println!("GalleryDL output: {:#?}", &gdl_output);
        }
        Ok(gdl_output)
    })
}

#[derive(Debug, Serialize, Deserialize, specta::Type, Clone)]
pub struct GalleryDlStatus {
    pub bytes_total: i64,
    pub bytes_downloaded: i64,
    pub bytes_per_second: i64,
    pub url_hash: String,
    pub url: String,
    pub extractor: String,
}

impl GalleryDlStatus {
    pub fn new_placeholder(url: &str) -> Self {
        Self {
            bytes_total: 0,
            bytes_downloaded: 0,
            bytes_per_second: 0,
            url_hash: hash_url(url),
            url: url.to_owned(),
            extractor: "".to_string(),
        }
    }
}

pub fn hash_url(url: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[derive(Debug, Serialize, Deserialize, Default, specta::Type)]
pub struct GalleryDlStatuses(HashMap<String, GalleryDlStatus>);

#[derive(Debug, Serialize, Deserialize)]
pub struct GalleryDlOutput {
    // might use extractor as a key for URL https://serde.rs/enum-representations.html
    pub extractor: String,
    pub base_url: String,
    pub url_extractors: Vec<URLExtractor>,
    pub dir_extractors: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct URLExtractor {
    //extractor: String, // Might be used as a key to put `meta`` into typed structs
    pub path: String,
    pub url: String,
    #[serde(flatten)]
    meta: Meta,
}

impl URLExtractor {
    pub fn extract_tags(
        &self,
        extractors: &Vec<&(dyn TagExtractor + Send + Sync)>,
    ) -> Result<Vec<ExtractedTag>> {
        match &self.meta {
            Meta::Other(value) => {
                let gdl_extractor = value["extractor"]
                    .as_str()
                    .ok_or(anyhow!("no extractor field found on the gallery_dl data"))?;

                let extractions = extractors
                    .iter()
                    .map(|e| e.extract_tags(gdl_extractor, value))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .flat_map(|e| e.flatten())
                    .collect::<Vec<_>>();

                Ok(extractions)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]

struct Configurable;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "extractor", content = "meta")]
enum Meta {
    // We don't have any typed structs for the extractor
    #[serde(untagged)]
    Other(Value),
}

#[derive(Debug, Error)]
pub enum PyError {
    #[error("RustPython exception, details: {0}")]
    PythonException(String),
    // This is a better way of handling errors instead of unwrapping errors inside errors, but i can't figure it out
    //#[error("Cannot get python Error")]
    //ErrorError,
}

#[pymodule]
mod rust_side {

    #[pyfunction]
    fn get_cert_path() -> String {
        let data_dir = dirs::data_dir().unwrap();
        let data_dir = data_dir.join("kasa");

        let cert_path = data_dir.join("cacert.pem");

        cert_path.to_str().unwrap().to_string()
    }
}

// fuck...
pub struct PyTrustMe(pub Interpreter);
unsafe impl Send for PyTrustMe {}
unsafe impl Sync for PyTrustMe {}
