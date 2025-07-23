use std::collections::HashMap;

use anyhow::{Ok, Result};
use extractors::configurable::{ExtractorConfig, extract_tags};
use log::trace;
use rustpython_pylib::FROZEN_STDLIB;
use rustpython_vm::{
    Interpreter, compiler::parser::ast::String, convert::ToPyObject, py_freeze, pymodule, vm,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
pub mod extractors;

const CERT_BYTES: &[u8] = include_bytes!("../cacert.pem");

pub fn init_interpreter() -> Interpreter {
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

    vm::Interpreter::with_init(Default::default(), |vm| {
        vm.add_native_modules(rustpython_stdlib::get_module_inits());

        vm.add_frozen(FROZEN_STDLIB);

        // First, just in case the order matters
        vm.add_native_module("rust_side", Box::new(rust_side::make_module));

        vm.add_frozen(py_freeze!(
            dir = "py/dependencies/gallery-dl/gallery_dl-1.30.0"
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
    interpreter: &Interpreter,
    url: &str,
    output_path: &str,
    gdl_config_path: Option<String>,
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

        dbg!(&gdl_config_path);

        let output = func
            .call(
                (url, output_path, gdl_config_path.unwrap_or("".to_string())),
                vm,
            )
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

        trace!("Raw gallery_dl output: {:#?}", &gdl_output);

        if std::env::var("KASA_GDL_OUTPUT").is_ok() {
            println!("GalleryDL output: {:#?}", &gdl_output);
        }
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
    meta: Meta,
}

impl URLExtractor {
    pub fn get_tags(
        &self,
        extractors: &HashMap<String, ExtractorConfig>,
    ) -> Result<Vec<ExtractedTag>> {
        match &self.meta {
            Meta::Other(value) => extract_tags(extractors, value),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExtractedTag {
    pub _type: String,
    pub name: String,
}

impl ExtractedTag {
    pub fn new(_type: &str, name: &str) -> Self {
        Self {
            _type: _type.to_string(),
            name: name.to_string(),
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
