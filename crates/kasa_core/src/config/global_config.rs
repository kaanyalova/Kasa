use std::{
    default, env,
    fs::{self, create_dir},
    path::{Path, PathBuf},
};

use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use toml_edit::{Array, DocumentMut, Value, value};

use crate::thumbnail::thumbnail_image::ThumbnailFormat;

const DEFAULT_CONFIG: &str = r#"
# Try to avoid using relative paths, they will cause problems, they should never be configured
# from the GUI anyways

[Database]
# Path of the currently open database file
db_path = ""
db_type = ""


[Thumbnails]
# Path of the db that stores the thumbnails
thumbs_db_path = "./thumbs.kasa"

# The max resolution for thumbnails, [width, height]
resolution = [256, 256]

# The file format for thumbnails 
# it can be one of "png", "jpeg", "webp_lossy", "webp_lossless"
thumbnail_format = "png"


[Downloader]
# Path that gallery_dl will output the extracted media
output_path = ""

# Optional: gallery_dl config path 
# gdl_config_path = "

[Layout]
show_filenames = false
thumbnail_scale = 1.5
"#;

#[derive(Serialize, Deserialize, Debug, PartialEq, specta::Type, Clone)]

pub struct Database {
    pub db_path: String,
    pub db_type: DatabaseType,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, specta::Type, Clone, Default)]

pub enum DatabaseType {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "remote")]
    Remote,
    #[serde(other)]
    #[default]
    Unknown,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            db_path: "".to_string(),
            db_type: DatabaseType::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, specta::Type, Clone)]

pub struct Thumbs {
    pub resolution: [u32; 2],
    pub thumbnail_format: ThumbnailFormat,
    pub thumbs_db_path: String,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, specta::Type, Clone)]
pub struct Downloader {
    pub output_path: String,
    // The plan was to have gallery-dl config options inside the config.toml
    // But toml_edit doesn't support serde types and i don't feel like manually parsing every single possible field
    // At least with this users might be able to bring their own config files

    // toml doesnt parse with Option<String> for some reason
    pub gdl_config_path: Option<String>,
}

impl Default for Thumbs {
    fn default() -> Self {
        Self {
            resolution: [256, 256],
            thumbnail_format: ThumbnailFormat::Png,
            thumbs_db_path: "./thumbs.kasa".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, specta::Type, Clone)]
pub struct Layout {
    show_filenames: bool,
    thumbnail_scale: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            show_filenames: false,
            thumbnail_scale: 1.5,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, specta::Type)]
pub struct GlobalConfig {
    #[serde(rename = "Database")]
    pub db: Database,
    #[serde(rename = "Thumbnails")]
    pub thumbs: Thumbs,
    #[serde(rename = "Downloader")]
    pub downloader: Downloader,
    #[serde(rename = "Layout")]
    pub layout: Layout,
}

pub fn get_config_dir() -> PathBuf {
    // Use config from env variables if present
    let from_env = env::var_os("KASA_CONFIG_DIR");

    if let Some(path) = from_env {
        return PathBuf::from(path.to_string_lossy().to_string());
    }

    let os_config_dir = dirs::config_dir().unwrap();

    os_config_dir.join("kasa")
}

pub fn get_config_impl() -> GlobalConfig {
    let path = get_config_dir().join("config.toml");
    find_or_create_config(&path);

    let f = fs::read_to_string(path).unwrap();

    let config: GlobalConfig = toml::from_str(&f).unwrap();
    config
}

pub fn get_tag_extractors_dir() -> Result<PathBuf> {
    let config_dir = get_config_dir();
    let extractor_dir = config_dir.join("extractors");

    if !&extractor_dir.exists() {
        std::fs::create_dir(&extractor_dir)?;
    }

    Ok(extractor_dir)
}

#[derive(specta::Type, Serialize, Deserialize)]
pub enum ResolutionKey {
    Width,
    Height,
}

/// Special function to set thumbnail resolution array keys
pub fn set_value_resolution(height: u32, width: u32) {
    let path = get_config_dir().join("config.toml");
    find_or_create_config(&path);

    let f = fs::read_to_string(&path).unwrap();

    let mut toml = f.parse::<DocumentMut>().unwrap();

    let vals = [width as i64, height as i64];
    toml["Thumbnails"]["resolution"] = value(Value::Array(Array::from_iter(vals)));

    fs::write(path, toml.to_string()).unwrap();
}

pub fn set_value(category: &str, key: &str, val: impl Into<Value>) {
    let path = get_config_dir().join("config.toml");

    find_or_create_config(&path);

    let f = fs::read_to_string(&path).unwrap();

    let mut toml = f.parse::<DocumentMut>().unwrap();

    toml[category][key] = value(val);

    fs::write(path, toml.to_string()).unwrap();
}

pub fn set_db_path_impl(db_path: &str) {
    let path = get_config_dir().join("config.toml");

    find_or_create_config(&path);

    let f = fs::read_to_string(&path).unwrap();

    let mut toml = f.parse::<DocumentMut>().unwrap();

    toml["Database"]["db_path"] = value(db_path);

    fs::write(path, toml.to_string()).unwrap();
}

pub fn set_db_type(db_type: DatabaseType) {
    let path = get_config_dir().join("config.toml");

    find_or_create_config(&path);

    let f = fs::read_to_string(&path).unwrap();

    let mut toml = f.parse::<DocumentMut>().unwrap();

    let db_type_str = match db_type {
        DatabaseType::Local => "local",
        DatabaseType::Remote => "remote",
        DatabaseType::Unknown => "unknown",
    };

    toml["Database"]["db_type"] = value(db_type_str);

    fs::write(path, toml.to_string()).unwrap();
}

pub fn set_thumbs_db_path_impl(db_path: &Path) {
    let path = get_config_dir().join("config.toml");

    find_or_create_config(&path);

    let f = fs::read_to_string(&path).unwrap();

    let mut toml = f.parse::<DocumentMut>().unwrap();

    toml["Thumbnails"]["thumbs_db_path"] = value(db_path.to_string_lossy().to_string());

    fs::write(path, toml.to_string()).unwrap();
}

/// Checks if the config file exists, creates it if it doesn't
/// `path` is absolute path to config.toml
fn find_or_create_config(path: &PathBuf) {
    // create the parent "kasa" directory if it doesn't exist
    let parent = path.parent().unwrap();

    if !parent.is_dir() {
        info!(
            "Config directory doesn't exist creating at {}",
            parent.display()
        );
        create_dir(parent).unwrap();
    }

    if !path.exists() {
        info!("Config file doesn't exist, creating at {}", path.display());
        fs::write(path, DEFAULT_CONFIG).unwrap()
    }
}

#[test]
fn default_config_parse() {
    #[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
    #[serde(deny_unknown_fields)]
    // Make sure this is the same as above, but with the `deny_unknown_fields`
    pub struct GlobalConfig {
        #[serde(rename = "Database")]
        pub db: Database,
        #[serde(rename = "Thumbnails")]
        pub thumbs: Thumbs,
        #[serde(rename = "Downloader")]
        pub downloader: Downloader,
        #[serde(rename = "Layout")]
        pub layout: Layout,
    }

    let config: GlobalConfig = toml::from_str(DEFAULT_CONFIG).unwrap();

    assert_eq!(config, GlobalConfig::default());
}

#[test]
fn test_config_creation() {
    let tempdir = tempfile::tempdir().unwrap().into_path();

    let config_path = tempdir.join("kasa").join("config.toml");

    find_or_create_config(&config_path);

    assert!(tempdir.join("kasa").is_dir());
    assert!(tempdir.join("kasa").join("config.toml").is_file());

    let config = fs::read_to_string(config_path).unwrap();
    let config_parsed: GlobalConfig = toml::from_str(&config).unwrap();
    let default_config_parsed: GlobalConfig = toml::from_str(DEFAULT_CONFIG).unwrap();

    assert_eq!(DEFAULT_CONFIG, config);

    // TODO why does it fail
    assert_eq!(default_config_parsed, config_parsed);
}
