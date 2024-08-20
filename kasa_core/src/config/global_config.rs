use std::{
    env,
    fs::{self, create_dir},
    path::PathBuf,
};

use log::info;
use serde::{Deserialize, Serialize};
use toml_edit::{value, DocumentMut};

const DEFAULT_CONFIG: &str = r#"
[Database]
# Path of the currently open database file
db_path = "./default.kasa"



[Thumbnails]
# Path of the db that stores the thumbnails
thumbs_db_path = "./thumbs.kasa"

# The max resolution for thumbnails
thumbnail_resolution = [256, 256]

# The file format for thumbnails
thumbnail_format = "png"

"#;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, specta::Type)]

pub struct Database {
    pub db_path: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, specta::Type)]

pub struct Thumbs {
    pub thumbnail_resolution: [u32; 2],
    pub thumbnail_format: ThumbnailFormat,
    pub thumbs_db_path: String,
}

impl Default for Thumbs {
    fn default() -> Self {
        Self {
            thumbnail_resolution: [256, 256],
            thumbnail_format: ThumbnailFormat::PNG,
            thumbs_db_path: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, specta::Type)]
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
    check_config(&path);

    let f = fs::read_to_string(path).unwrap();

    let config: GlobalConfig = toml::from_str(&f).unwrap();
    config
}

pub fn set_value(category: &str, key: &str, val: &str) {
    let path = get_config_dir().join("config.toml");

    check_config(&path);

    let f = fs::read_to_string(&path).unwrap();

    let mut toml = f.parse::<DocumentMut>().unwrap();

    toml[category][key] = value(val);

    fs::write(path, &toml.to_string()).unwrap();
}

/// Checks if the config file exists, creates it if it doesn't
/// `path` is absolute path to config.toml
fn check_config(path: &PathBuf) {
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
    }

    let config: GlobalConfig = toml::from_str(DEFAULT_CONFIG).unwrap();

    assert_eq!(config, GlobalConfig::default());
}

#[test]
fn test_config_creation() {
    let tempdir = tempfile::tempdir().unwrap().into_path();

    let config_path = tempdir.join("kasa").join("config.toml");

    check_config(&config_path);

    assert!(tempdir.join("kasa").is_dir());
    assert!(tempdir.join("kasa").join("config.toml").is_file());

    let config = fs::read_to_string(config_path).unwrap();
    let config_parsed: GlobalConfig = toml::from_str(&config).unwrap();
    let default_config_parsed: GlobalConfig = toml::from_str(DEFAULT_CONFIG).unwrap();

    assert_eq!(DEFAULT_CONFIG, config);

    // TODO why does it fail
    assert_eq!(default_config_parsed, config_parsed);
}
