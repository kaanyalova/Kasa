use std::path::Path;

use anyhow::{Context, Result};
use oar_ocr::{
    oarocr::{OAROCRBuilder, TextRegion},
    utils::load_image,
};

use super::{ModelManager, ModelManagerModels};
use crate::model_sources::{Model, OcrModel};

static OCR_IMAGE_BATCH_COUNT: usize = 10;

impl ModelManager {
    /// returns the vector of text regions while keeping the original indices of the input image_paths
    /// if something goes wrong while ocring the image it returns None for that index
    pub fn ocr_images(
        &self,
        model_name: &str,
        image_paths: Vec<&Path>,
    ) -> Result<Vec<Option<Vec<TextRegion>>>> {
        assert!(self.is_onnx_rt_loaded);
        let model = self
            .models
            .get_downloaded_ocr_model(model_name)
            .context(format!("the model with name {} doesn't exist", model_name))?;

        let paths = &model.paths;
        let ocr = OAROCRBuilder::new(
            &paths.det_model_path,
            &paths.rec_model_path,
            &paths.dict_path,
        )
        .build()?;

        let mut all_results = Vec::with_capacity(image_paths.len());

        for image_batch in image_paths.chunks(OCR_IMAGE_BATCH_COUNT) {
            let (indices_of_original_array, valid_images): (Vec<_>, Vec<_>) = image_batch
                .iter()
                .enumerate()
                .filter(|(_i, p)| p.exists())
                .filter_map(|(i, p)| load_image(p).ok().map(|img| (i, img)))
                .unzip();

            let results = ocr.predict(valid_images)?;

            let mut mapped_results = vec![None; image_batch.len()];

            // map them back to a new array so it keeps the original indices and invalid images are None
            for (orig_idx, result) in indices_of_original_array.into_iter().zip(results) {
                mapped_results[orig_idx] = Some(result);
            }

            all_results.extend(mapped_results);
        }

        // lets just serialize whatever oarocr outputs later, it derives serialize
        let text_regions: Vec<Option<Vec<TextRegion>>> = all_results
            .into_iter()
            .map(|r| r.map(|r| r.text_regions))
            .collect();

        Ok(text_regions)
    }
}

impl ModelManagerModels {
    pub async fn download_ocr_model(&mut self, name: &str) -> Result<()> {
        let model_source = self
            .model_sources
            .ocr_models
            .get(name)
            .context(format!("the model with name {} doesn't exist", name))?;

        let mut model = model_source.clone().into_model(&self.models_base_path);

        model
            .download_model(&self.reqwest, self.on_progress.as_ref())
            .await?;

        Ok(())
    }

    pub fn get_downloaded_ocr_model(&self, name: &str) -> Option<OcrModel> {
        let model_source = self.model_sources.ocr_models.get(name)?;

        let mut model = model_source.clone().into_model(&self.models_base_path);

        if model.check_if_already_downloaded() {
            Some(model)
        } else {
            None
        }
    }

}
