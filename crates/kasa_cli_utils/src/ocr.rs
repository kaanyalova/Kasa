use std::path::Path;

use anyhow::{Context, Result};
use kasa_ai::model_manager::{self, ModelManager};

pub async fn show_ocr_image_info(
    file_path: &Path,
    rt_path: &Path,
    models_path: &Path,
    model_name: &str,
) -> Result<()> {
    let mut model_manager = ModelManager::new(
        rt_path,
        models_path,
        Box::new(|p| {
            //dbg!(p);
        }),
    );

    model_manager.load_default_sources()?;
    model_manager.load_onnx_rt()?;

    let is_downloaded = model_manager
        .models
        .get_downloaded_ocr_model(model_name)
        .is_some();

    if !is_downloaded {
        println!("the model {} is not downloaded, downloading...", model_name);
        model_manager.models.download_ocr_model(model_name).await?;
    }

    let ocr = model_manager.ocr_images(model_name, vec![file_path])?;
    let ocred = ocr
        .first()
        .context("ocr_images returned no results")?
        .as_ref()
        .context("failed to OCR the image")?;

    dbg!(&ocred);

    Ok(())
}
