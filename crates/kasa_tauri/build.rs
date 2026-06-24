//use kasa_server::api::get_openapi_spec;
use std::io::Cursor;

fn main() {
    /*
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
    */

    //let rt = tokio::runtime::Runtime::new().unwrap();
    //let spec = rt.block_on(async { get_openapi_spec().await });
    //let spec = serde_json::from_str(&spec).unwrap();
    //let mut generator = progenitor::Generator::default();
    //let tokens = generator.generate_tokens(&spec).unwrap();
    //let ast = syn::parse2(tokens).unwrap();
    //let code = prettyplease::unparse(&ast);
    //std::fs::write("../../src-tauri/src/openapi.rs", code).unwrap();

    tauri_build::build()
}
