mod dump_random_gi_layout;
mod index_folder;
mod populate_tags;
mod thumbnail;

use clap::Parser;
use dump_random_gi_layout::dump_random_gi_layout;
use index_folder::index_folder;
use populate_tags::populate_tags;
//use thumbnail::thumbnail;

#[derive(Parser)] // requires `derive` feature
#[command(name = "kasa-cli")]
#[command(bin_name = "kasa-cli")]
enum KasaCli {
    PopulateTags(PopulateTagsArgs),
    IndexFolder(IndexFolderArgs),
    DumpGILayout,
    //ThumbnailFolder(ThumbnailArgs),
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
    env_logger::init();
    dotenvy::dotenv().unwrap();
    let args = KasaCli::parse();
    match args {
        KasaCli::PopulateTags(args) => populate_tags(args).await,
        KasaCli::IndexFolder(args) => index_folder(args).await,
        KasaCli::DumpGILayout => dump_random_gi_layout(),
    }
}
