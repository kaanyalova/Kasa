use anyhow::{anyhow, Ok, Result};
use extractors::danbooru::Danbooru;
use rustpython_pylib::FROZEN_STDLIB;
use rustpython_vm::{
    compiler::parser::ast::String,
    convert::{ToPyObject, ToPyResult},
    py_freeze, vm, Interpreter, Py,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
mod extractors;

pub fn init_interpreter() -> Interpreter {
    vm::Interpreter::with_init(Default::default(), |vm| {
        vm.add_native_modules(rustpython_stdlib::get_module_inits());
        vm.add_frozen(FROZEN_STDLIB);
        vm.add_frozen(py_freeze!(
            dir = "py/dependencies/gallery-dl/gallery_dl-1.27.7"
        ));
        vm.add_frozen(py_freeze!(
            dir = "py/dependencies/charset_normalizer/charset_normalizer-3.4.0"
        ));
        vm.add_frozen(py_freeze!(dir = "py/dependencies/idna/idna-3.10"));
        vm.add_frozen(py_freeze!(
            dir = "py/dependencies/requests/requests-2.32.3/src"
        ));
        vm.add_frozen(py_freeze!(dir = "py/dependencies/certifi"));
        vm.add_frozen(py_freeze!(
            dir = "py/dependencies/urllib3/urllib3-2.2.3/src"
        ));
        vm.add_frozen(py_freeze!(dir = "py/py_src"));
    })
}

pub fn gdl_download(
    interpreter: Interpreter,
    url: &str,
    output_path: &str,
) -> Result<GalleryDlOutput> {
    interpreter.enter(|vm| {
        let module = vm.import("gdl", 0).map_err(|e| {
            PyError::PythonException(
                e.to_pyobject(vm).try_into_value::<String>(vm).unwrap(), //.unwrap_or("Cannot get python error message!".into()),
            )
        })?;
        let func = module.get_attr("download", vm).map_err(|e| {
            PyError::PythonException(
                e.to_pyobject(vm).try_into_value::<String>(vm).unwrap(), //.unwrap_or("Cannot get python error message!".to_string()),
            )
        })?;

        let output = func
            .call((url, output_path), vm)
            .map_err(|e| {
                vm.print_exception(e)

                //PyError::PythonException(
                //    e.to_pyobject(vm).try_into_value::<String>(vm).unwrap(), //.unwrap_or("Cannot get python error message!".to_string()),
                //)
            })
            .unwrap();

        let output: String = output.try_into_value(vm).map_err(|e| {
            PyError::PythonException(
                e.to_pyobject(vm).try_into_value::<String>(vm).unwrap(), //.unwrap_or("Cannot get python error message!".to_string()),
            )
        })?;

        let gdl_output: GalleryDlOutput = serde_json::from_str(&output)?;

        Ok(gdl_output)
    })
}

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
    pub meta: Meta,
}

impl URLExtractor {
    pub fn get_tags(&self) -> Vec<ExtractedTag> {
        match &self.meta {
            Meta::Danbooru(meta) => meta.tags(),
            Meta::Other(_) => vec![],
        }
    }
}

#[derive(Debug)]
pub struct ExtractedTag {
    pub _type: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "extractor", content = "meta")]
enum Meta {
    #[serde(rename = "danbooru")]
    Danbooru(Danbooru),
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
