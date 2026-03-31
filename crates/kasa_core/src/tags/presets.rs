use std::u32;

use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, query, query_as, query_scalar};

use crate::db::schema::TagPreset;

#[derive(Deserialize, Debug, Serialize, Clone, specta::Type)]
pub struct TagPresetData {
    included_tags: Vec<String>,
    excluded_tags: Vec<String>,
}

pub async fn new_or_update_preset_impl(
    includes: Vec<String>,
    excludes: Vec<String>,
    name: &str,
    pool: &Pool<Sqlite>,
) -> Result<()> {
    let data = TagPresetData {
        included_tags: includes,
        excluded_tags: excludes,
    };

    query("INSERT OR REPLACE INTO SearchPreset(name, preset) VALUES ? ?")
        .bind(name)
        .bind(serde_json::to_string(&data).unwrap())
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_preset_impl(name: &str, pool: Pool<Sqlite>) -> Result<TagPresetData> {
    let preset_json: String = query_scalar("SELECT preset FROM SearchPreset WHERE name = ?")
        .bind(name)
        .fetch_one(&pool)
        .await?;

    let preset: TagPresetData = serde_json::from_str(&preset_json)?;

    Ok(preset)
}

#[derive(Deserialize, Debug, Serialize, Clone, specta::Type)]
pub struct PresetListEntry {
    pub name: String,
    include_count: u32,
    exclude_count: u32,
}

pub async fn get_preset_list_impl(pool: &Pool<Sqlite>) -> Result<Vec<PresetListEntry>> {
    let entries: Vec<TagPreset> = query_as("SELECT name, preset FROM SearchPreset")
        .fetch_all(pool)
        .await?;

    let mut result = vec![];

    let preset_list: Vec<PresetListEntry> = entries
        .iter()
        .map(|e| PresetListEntry {
            name: e.name.clone(),
            include_count: e.preset.included_tags.len() as u32,
            exclude_count: e.preset.excluded_tags.len() as u32,
        })
        .collect();

    Ok(result)
}
