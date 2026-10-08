pub use ort::session::Session;

pub mod image_embeddings;
pub mod model_manager;
pub mod model_sources;
pub mod wdv_tagger;

pub use oar_ocr::oarocr::TextRegion;
pub use oar_ocr::processors::{BoundingBox, Point};

pub fn prepare_session(model_path: &str) -> Session {
    let onnx_path = std::env::var("KASA_ONNX_RT_PATH").unwrap();
    ort::init_from(&onnx_path)
        .unwrap()
        .with_execution_providers([])
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
