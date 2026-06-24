use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub enum Args {
    #[command(name = "run")]
    StartServer(ServerArgs),
    #[command(name = "openapi")]
    WriteOpenApiSpec(OpenApiArgs),
}

#[derive(Debug, Parser, Default)]
pub struct ServerArgs {
    pub db_path: Option<PathBuf>,
    #[arg(long, short, default_value_t = {"127.0.0.1".to_string()})]
    pub ip_address: String,
    #[arg(long, short, default_value_t = 8123)]
    pub port: u16,
    #[arg(long, short)]
    pub thumbs_db_path: Option<PathBuf>,
}

#[derive(Debug, Parser)]
pub struct OpenApiArgs {
    #[arg(long, short)]
    pub path: Option<PathBuf>,
}
