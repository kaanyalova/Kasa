use anyhow::{Ok, Result};
use std::{
    path::Path,
    time::Instant,
};

use fastembed::{ImageEmbedding, ImageInitOptions, InitOptions, TextEmbedding};

// takes ~100ms per image on a 9950x3d
pub fn generate_image_embedding_single(path: &Path) {
    let mut model = ImageEmbedding::try_new(
        ImageInitOptions::new(fastembed::ImageEmbeddingModel::NomicEmbedVisionV15)
            .with_show_download_progress(true),
    )
    .unwrap();

    let start = Instant::now();

    let embeddings = model.embed(vec![path], None).unwrap();

    println!("Embeddings length: {}", embeddings.len()); // -> Embeddings length: 2
    println!("Embedding dimension: {}", embeddings[0].len()); // -> Embedding dimension: 512
    //println!("Data : {:?}", embeddings[0]);

    let elapsed = start.elapsed();
    println!("Took {}ms", elapsed.as_millis());
}

pub fn generate_image_embeddings(paths: Vec<&Path>) -> Result<Vec<Vec<f32>>> {
    let mut model = ImageEmbedding::try_new(
        ImageInitOptions::new(fastembed::ImageEmbeddingModel::NomicEmbedVisionV15)
            .with_show_download_progress(true),
    )?;

    let embeddings = model.embed(paths, None)?;
    Ok(embeddings)
}

pub fn generate_text_embedding_single(text: &str) {
    let mut model = TextEmbedding::try_new(
        InitOptions::new(fastembed::EmbeddingModel::NomicEmbedTextV15)
            .with_show_download_progress(true),
    )
    .unwrap();

    let start = Instant::now();

    let embeddings = model.embed(vec![text], None).unwrap();

    println!("Embeddings length: {}", embeddings.len()); // -> Embeddings length: 2
    println!("Embedding dimension: {}", embeddings[0].len()); // -> Embedding dimension: 512
    println!("Data : {:?}", embeddings[0]);

    let elapsed = start.elapsed();
    println!("Took {}ms", elapsed.as_millis());
}

#[cfg(feature = "qwen3")]

// this is like really, really slow on cpu, almost 14 seconds on a 9950x3d, is the candle implementation slow
// might try out llama.cpp
pub fn qwen3_generate_image_embeddings_single(path: &Path) {
    use candle_core::{DType, Device};
    use fastembed::Qwen3VLEmbedding;

    let device = Device::Cpu;

    let model =
        Qwen3VLEmbedding::from_hf("Qwen/Qwen3-VL-Embedding-2B", &device, DType::F32, 2048).unwrap();

    println!("Running embed process");

    let start = Instant::now();

    let image_embeddings = model.embed_images(&vec![path]).unwrap();

    println!("Image embeddings length: {}", image_embeddings.len());
    println!("Embedding dimension: {}", image_embeddings[0].len());

    let elapsed = start.elapsed();
    println!("Took {}ms", elapsed.as_millis());
}
