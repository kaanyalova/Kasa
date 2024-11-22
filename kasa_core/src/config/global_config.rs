use std::{
    env,
    fs::{self, create_dir},
    path::PathBuf,
    str::FromStr,
};

use anyhow::Result;
use log::info;
use rayon::vec;
use rustpython_vm::PyObject;
use rustpython_vm::{
    pyclass, pymodule, PyPayload, PyResult, TryFromBorrowedObject, VirtualMachine,
};
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;
use toml_edit::{de::from_document, value, Array, DocumentMut, Value};

const DEFAULT_CONFIG: &str = r#"
# Try to avoid using relative paths, they will cause problems, they should never be configured
# from the GUI anyways

[Database]
# Path of the currently open database file
db_path = "./default.kasa"


[Thumbnails]
# Path of the db that stores the thumbnails
thumbs_db_path = "./thumbs.kasa"

# The max resolution for thumbnails, [width, height]
resolution = [256, 256]

# The file format for thumbnails
thumbnail_format = "png"


[Downloader]
# Path that gallery_dl will output the extracted media
output_path = ""

# Optional: gallery_dl config path 
# gdl_config_path = "
"#;

#[derive(Serialize, Deserialize, Debug, PartialEq, specta::Type)]

pub struct Database {
    pub db_path: String,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            db_path: "./default.kasa".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, specta::Type)]

pub struct Thumbs {
    pub resolution: [u32; 2],
    pub thumbnail_format: ThumbnailFormat,
    pub thumbs_db_path: String,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, specta::Type)]
pub struct Downloader {
    pub output_path: String,
    // The plan was to have gallery-dl config options inside the config.toml
    // But toml_edit doesn't support serde types and i don't feel like manually parsing every single possible field
    // At least with this users might be able to bring their own config files

    // toml doesnt parse with Option<String> for some reason
    pub gdl_config_path: String,
}

impl Default for Thumbs {
    fn default() -> Self {
        Self {
            resolution: [256, 256],
            thumbnail_format: ThumbnailFormat::PNG,
            thumbs_db_path: "./thumbs.kasa".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, specta::Type, strum::IntoStaticStr)]
#[serde(rename_all = "lowercase")]
pub enum ThumbnailFormat {
    PNG,
    JPEG,
    AVIF,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, specta::Type)]
pub struct GlobalConfig {
    #[serde(rename = "Database")]
    pub db: Database,
    #[serde(rename = "Thumbnails")]
    pub thumbs: Thumbs,
    #[serde(rename = "Downloader")]
    pub downloader: Downloader,
}

fn get_config_dir() -> PathBuf {
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

pub fn get_configurable_tag_extractor_path() -> Result<PathBuf> {
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

    fs::write(path, &toml.to_string()).unwrap();
}

/// Sets a string value for given category and key
pub fn set_value_str(category: &str, key: &str, val: &str) {
    let path = get_config_dir().join("config.toml");

    find_or_create_config(&path);

    let f = fs::read_to_string(&path).unwrap();

    let mut toml = f.parse::<DocumentMut>().unwrap();

    toml[category][key] = value(val);

    fs::write(path, &toml.to_string()).unwrap();
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
        fs::write(&path, DEFAULT_CONFIG).unwrap()
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
    }

    let config: GlobalConfig = toml::from_str(DEFAULT_CONFIG).unwrap();

    assert_eq!(config, GlobalConfig::default());
}

#[test]
fn test_config_creation() {
    let tempdir = tempfile::tempdir().unwrap().into_path();

    let config_path = tempdir.join("kDebugasa").join("config.toml");

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
