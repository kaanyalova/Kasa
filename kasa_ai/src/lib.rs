use ort::execution_providers::CUDAExecutionProvider;
use ort::execution_providers::ROCmExecutionProvider;

pub use ort::session::Session;

pub mod wdv_tagger;

pub fn prepare_session(model_path: &str) -> Session {
    let onnx_path = std::env::var("KASA_ONNX_RT_PATH").unwrap();
    ort::init_from(&onnx_path)
        .unwrap()
        .with_execution_providers([
            CUDAExecutionProvider::default().build(),
            ROCmExecutionProvider::default().build(),
        ])
        .commit();

    Session::builder()
        .unwrap()
        .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
        .unwrap()
        .with_intra_threads(16)
        .unwrap()
        .commit_from_file(model_path)
        .unwrap()
}
