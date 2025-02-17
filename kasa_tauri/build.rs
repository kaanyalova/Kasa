use std::io::{BufReader, Cursor};

fn main() {
    const RUFFLE_PACKAGE_URL: &str = "https://github.com/ruffle-rs/ruffle/releases/download/nightly-2025-02-10/ruffle-nightly-2025_02_10-web-selfhosted.zip";

    let bytes = reqwest::blocking::get(RUFFLE_PACKAGE_URL)
        .unwrap()
        .bytes()
        .unwrap();

    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    archive.extract("../static/ruffle").unwrap();

    tauri_build::build()
}
