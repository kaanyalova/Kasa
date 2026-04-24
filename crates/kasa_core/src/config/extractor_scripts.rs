use anyhow::Result;
use sqlx::{Pool, Sqlite, query_scalar};
use std::{fs, path::PathBuf};

use crate::config::global_config::get_tag_extractors_dir;

const DEFAULT_PYTHON_EXTRACTOR: &str = include_str!("default_extractor_script.py");

pub fn create_or_get_path_for_extractor_impl(
    extractor_name: &str,
    file_extension: &str,
) -> Result<PathBuf> {
    let extractors_dir = get_tag_extractors_dir()?;
    let extractor_path = extractors_dir.join(format!("{}.{}", extractor_name, file_extension));

    if !extractor_path.exists() {
        fs::File::create_new(&extractor_path)?;
    }

    Ok(extractor_path)
}

pub fn create_or_get_extractor_contents_impl(
    extractor_name: &str,
    file_extension: &str,
) -> Result<String> {
    let path = create_or_get_path_for_extractor_impl(extractor_name, file_extension)?;
    std::fs::write(&path, DEFAULT_PYTHON_EXTRACTOR)?;
    Ok(fs::read_to_string(&path)?)
}

pub async fn get_existing_extractor_names_impl(pool: &Pool<Sqlite>) -> Result<Vec<String>> {
    let extractors_dir = get_tag_extractors_dir()?;
    let paths: Vec<String> = fs::read_dir(extractors_dir)?
        .filter_map(|f| f.ok())
        .filter(|f| f.path().extension().is_some_and(|ext| ext == "py"))
        .filter_map(|f| f.file_name().into_string().ok())
        .collect();

    let found_in_db: Vec<String> = query_scalar("SELECT source FROM MediaSource GROUP BY source")
        .fetch_all(pool)
        .await?;

    let mut all = paths;

    all.extend(found_in_db);
    all.sort();
    all.dedup();

    Ok(all)
}

pub async fn get_example_metadata_for_extractor_impl(
    pool: &Pool<Sqlite>,
    name: &str,
) -> Result<String> {
    let example_metadata: Option<String> =
        query_scalar("SELECT raw_data FROM MediaSource WHERE source = ?")
            .bind(name)
            .fetch_optional(pool)
            .await?;

    let placeholder = r#"{"error" : "No example metadata for the selected extractor found, try downloading something with this extractor first."}"#.to_owned();

    Ok(example_metadata.unwrap_or(placeholder))
}
