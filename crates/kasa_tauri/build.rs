use std::io::Cursor;

fn main() {
    const RUFFLE_PACKAGE_URL: &str = "https://github.com/ruffle-rs/ruffle/releases/download/nightly-2025-02-10/ruffle-nightly-2025_02_10-web-selfhosted.zip";

    let bytes = reqwest::blocking::get(RUFFLE_PACKAGE_URL)
        .unwrap()
        .bytes()
        .unwrap();

    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    archive.extract("../../static/ruffle").unwrap();

    const PDFJS_PACKAGE_URL: &str =
        "https://github.com/mozilla/pdf.js/releases/download/v5.4.54/pdfjs-5.4.54-dist.zip";

    let bytes = reqwest::blocking::get(PDFJS_PACKAGE_URL)
        .unwrap()
        .bytes()
        .unwrap();

    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    archive.extract("../../static/pdfjs").unwrap();

    tauri_build::build()
}
