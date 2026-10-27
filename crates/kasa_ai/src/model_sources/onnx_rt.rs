use std::{
    env,
    fs::{self, File},
    io::Cursor,
    path::{Path, PathBuf},
};

use anyhow::Result;
use libloading::Library;
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

use crate::model_sources::{Model, ModelDownloadProgress, ModelSourcesConfig, ModelState};

// onnx rt supports a billion different architectures, but provides builds only for cpu and cuda
// and others are a pain to build
//
// theres no vulkan execution provider that would work everywhere (except apple, lol)
// either so you better have an nvidia gpu if you want gpu acceleration
//
// theres a webgpu backend but thats really slow (slower than cpu execution) and requires building Dawn
//
// https://github.com/microsoft/onnxruntime/issues/21917
//
// lets hope they add vulkan support some day so I can get rid of this cuda and dynamic loading
// shit and have runtimes that i can statically link to without requiring 10s of gbs of cuda
// libraries and without fetching them at runtime
//
pub enum OnnxRtArchitecture {
    LinuxX64,
    LinuxX64Cuda13,
    LinuxX64Cuda12,
    LinuxArm64,

    WindowsX64,
    WindowsArm64,
    WindowsArm64X,
    WindowsX64Cuda12,
    WindowsX64Cuda13,

    OsxArm64,
}

pub struct OnnxRtLibraries {
    pub sources: Vec<OnnxRTSource>,
    pub filtered_sources: Vec<OnnxRTSource>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnnxRTSource {
    pub version_reference: String,
    pub os: String,
    pub arch: String,
    pub execution_provider_name: String,
    pub is_cuda: bool,
    pub uses_gpu: bool,
    pub execution_provider_version: Option<String>,
    pub url: String,
    pub sha256: String,
}

impl OnnxRTSource {
    pub fn into_model(self, base_path: &Path) -> OnnxRT {
        let base_dir = base_path
            .join("onnx_rt")
            .join(&self.os)
            .join(&self.arch)
            .join(&self.version_reference)
            .join(&self.execution_provider_name)
            .join(
                self.execution_provider_version
                    .clone()
                    .unwrap_or("".to_string()),
            );

        #[cfg(target_family = "unix")]
        let onnxrt_lib_name = "libonnxruntime.so";
        #[cfg(target_os = "windows")]
        let onnxrt_lib_name = "libonnxruntime.dll";

        let onnxrt_lib_path = base_dir.join("lib").join(onnxrt_lib_name);

        let paths = OnnxRTPaths {
            runtime_base_dir: base_dir,
            runtime_dynamic_lib_path: onnxrt_lib_path,
        };

        OnnxRT {
            state: ModelState::NotDownloaded,
            paths,
            source: self,
        }
    }
}

pub struct OnnxRTPaths {
    runtime_base_dir: PathBuf,
    runtime_dynamic_lib_path: PathBuf,
}

pub struct OnnxRT {
    state: ModelState,
    pub paths: OnnxRTPaths,
    pub source: OnnxRTSource,
}

#[async_trait::async_trait]
impl Model for OnnxRT {
    async fn download_model(
        &mut self,
        client: &reqwest::Client,
        on_progress: impl Fn(ModelDownloadProgress) + Send + Sync,
    ) -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let filename = format!("onnxrt_{}.zip", self.source.sha256);
        let temp_file = tempdir.path().join(filename);

        Self::streaming_download(client, &self.source.url, &temp_file, &on_progress).await?;

        on_progress(ModelDownloadProgress::HashingStart);
        Self::check_hash(&temp_file, &self.source.sha256)?;
        on_progress(ModelDownloadProgress::HashingEnd);

        on_progress(ModelDownloadProgress::UnzippingStart);
        let file = File::open(&temp_file)?;
        let mut archive = zip::ZipArchive::new(file)?;
        archive.extract(&self.paths.runtime_base_dir)?;
        on_progress(ModelDownloadProgress::UnzippingEnd);

        self.state = ModelState::Downloaded;
        Ok(())
    }

    fn check_and_update_status(&mut self) {
        let exists =
            self.paths.runtime_base_dir.is_dir() && self.paths.runtime_dynamic_lib_path.exists();

        if exists {
            self.state = ModelState::Downloaded
        }
    }

    fn status(&self) -> ModelState {
        self.state.clone()
    }
}

pub fn possibly_get_cuda_version() -> Option<i32> {
    #[cfg(target_os = "linux")]
    let possible_runtime_paths = ["libcudart.so.13", "libcudart.so.12", "libcudart.so"];
    #[cfg(target_os = "windows")]
    let possible_runtime_paths = ["cudart64_13.dll", "cudart64_12.dll"];

    for possible_runtime in possible_runtime_paths {
        unsafe {
            let library = Library::new(possible_runtime);

            if let Ok(library) = library {
                use std::ffi::c_int;

                let mut out: c_int = 0;
                // the result is probably 32 bits
                // https://docs.nvidia.com/cuda/archive/9.0/cuda-runtime-api/group__CUDART____VERSION.html
                let version_function = {
                    library
                        .get::<unsafe extern "C" fn(*mut c_int) -> c_int>(b"cudaRuntimeGetVersion")
                };

                if let Ok(version_function) = version_function {
                    let cuda_result = version_function(&mut out);
                    if out != 0 && cuda_result == 0 {
                        return Some(out);
                    }
                }
            }
        }
    }

    None
}

fn get_major_cuda_version(version: i32) -> i32 {
    version / 1000
}

impl OnnxRtLibraries {
    pub fn new(sources: Vec<OnnxRTSource>) -> Result<Self> {
        let libraries = Self {
            sources,
            filtered_sources: vec![],
        };
        Ok(Self {
            sources: libraries.filter_available_architecture_sources()?,
            filtered_sources: vec![],
        })
    }

    fn filter_available_architecture_sources(&self) -> Result<Vec<OnnxRTSource>> {
        let arch = env::consts::ARCH;
        let os = env::consts::OS;
        let gpus = sysinfo::Gpus::new()?;
        let gpus = gpus.list();
        let has_nvidia_gpu = gpus.iter().any(|g| g.vendor() == Some("nvidia"));
        let cuda_version = if has_nvidia_gpu {
            possibly_get_cuda_version().map(get_major_cuda_version)
        } else {
            None
        };

        let available_sources: Vec<OnnxRTSource> = self
            .sources
            .iter()
            .filter(|source| {
                source.os == os
                    && source.arch == arch
                    && source.is_cuda == has_nvidia_gpu
                    && (source.execution_provider_version == cuda_version.map(|v| v.to_string())
                        || source.execution_provider_version.is_none())
            })
            .cloned()
            .collect();

        Ok(available_sources)
    }

    pub fn load_sources(&mut self, sources: Vec<OnnxRTSource>) -> Result<()> {
        self.sources = sources;
        self.filter_available_architecture_sources()?;
        Ok(())
    }

    pub fn load_sources_from_toml_path(&mut self, path: &Path) -> Result<()> {
        let file = fs::read_to_string(path)?;
        let config: ModelSourcesConfig = toml::from_str(&file)?;
        self.load_sources(config.onnx_rt_sources)?;

        Ok(())
    }
}
