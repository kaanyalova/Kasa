use std::path::Path;

use fastembed::{ImageEmbedding, ImageInitOptions};

pub fn generate_embeddings(path: &Path) {
    let mut model = ImageEmbedding::try_new(
        ImageInitOptions::new(fastembed::ImageEmbeddingModel::NomicEmbedVisionV15)
            .with_show_download_progress(true),
    )
    .unwrap();

    let embeddings = model.embed(vec![path], None).unwrap();

    println!("Embeddings length: {}", embeddings.len()); // -> Embeddings length: 2
    println!("Embedding dimension: {}", embeddings[0].len()); // -> Embedding dimension: 512
    println!("Data : {:?}", embeddings[0]);
}
