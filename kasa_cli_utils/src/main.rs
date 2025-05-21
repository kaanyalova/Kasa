mod ai_tag_images;
mod dump_random_gi_layout;
mod gdl;
mod index_all_ai_images;
mod index_folder;
mod nuke_db_versioning;
mod populate_tags;
mod thumbnail;

use ai_tag_images::ai_tag_images;
use clap::Parser;
use dump_random_gi_layout::dump_random_gi_layout;
use gdl::gdl;
use index_all_ai_images::index_all_ai_images;
use index_folder::index_folder;
use kasa_core::config::global_config::get_configurable_tag_extractor_path;
use kasa_python::extractors::configurable::get_extractors_from_path;
use nuke_db_versioning::nuke_db_versioning;
use populate_tags::populate_tags;
//use thumbnail::thumbnail;

#[derive(Parser)] // requires `derive` feature
enum KasaCli {
    PopulateTags(PopulateTagsArgs),
    #[command(alias = "index")]
    IndexFolder(IndexFolderArgs),
    DumpGILayout,
    #[command(alias = "get-ai")]
    GetAiTags(IndexAllAIImagesArgs), //ThumbnailFolder(ThumbnailArgs),
    #[command(alias = "gdl")]
    GalleryDL(GalleryDlArgs),
    #[command(alias = "nuke")]
    NukeDBVersioning,
    #[command(alias = "tag-ai")]
    /// Needs the following environment variables set
    ///
    /// KASA_ONNX_RT_PATH: Path to the libonnxruntime.so or onnxruntime.dll
    /// KASA_WDV_MODEL_PATH: Path to the WDV Model https://huggingface.co/SmilingWolf
    /// KASA_WDV_LABEL_PATH: Path to the WDV Model labels
    TagUsingAi,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct GalleryDlArgs {
    url: String,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct IndexAllAIImagesArgs {
    #[clap(default_value_t = 50)]
    #[arg(long)]
    tag_max_len: u64,

    #[arg(long)]
    db_path: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct PopulateTagsArgs {
    #[arg(long)]
    tags_path: std::path::PathBuf,
    #[arg(long)]
    db_path: std::path::PathBuf,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct IndexFolderArgs {
    #[arg(long)]
    folder: std::path::PathBuf,
    #[arg(long)]
    #[clap(default_value_t = 8)]
    cores: i64,
    #[arg(long)]
    db_path: Option<std::path::PathBuf>,
    #[arg(long)]
    thumbs_db_path: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = true)]
    use_config_file: bool,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct ThumbnailArgs {
    #[arg(long)]
    in_path: std::path::PathBuf,
    #[arg(long)]
    out_path: std::path::PathBuf,
}

#[tokio::main]
async fn main() {
    let extractor_path = get_configurable_tag_extractor_path().unwrap();
    let extractors = get_extractors_from_path(&extractor_path.to_string_lossy()).unwrap();

    env_logger::init();
    dotenvy::dotenv().unwrap();
    let args = KasaCli::parse();
    match args {
        KasaCli::PopulateTags(args) => populate_tags(args).await,
        KasaCli::IndexFolder(args) => index_folder(args).await,
        KasaCli::DumpGILayout => dump_random_gi_layout(),
        KasaCli::GetAiTags(args) => {
            index_all_ai_images(args.db_path, args.tag_max_len as usize).await
        }
        KasaCli::GalleryDL(args) => gdl(&args.url, extractors).await,
        KasaCli::NukeDBVersioning => nuke_db_versioning().await,
        KasaCli::TagUsingAi => ai_tag_images().await,
    }
}
