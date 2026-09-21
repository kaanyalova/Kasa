use std::sync::Arc;

use kasa_core::config::global_config::GlobalConfig;

use crate::clients::{KasaClient, local::LocalKasaClient, remote::RemoteKasaClient};

uniffi::setup_scaffolding!();

pub mod clients;
pub mod errors;
pub mod events;

#[uniffi::export]
async fn new_local(config: &GlobalConfig) -> Arc<dyn KasaClient> {
    let mut client = LocalKasaClient::wait_for_frontend();
    client.initialize(&config).await;
    Arc::new(client)
}

#[uniffi::export]
async fn new_remote(config: &GlobalConfig) -> Arc<dyn KasaClient> {
    let mut client = RemoteKasaClient::wait_for_frontend();
    client.initialize(&config).await;
    Arc::new(client)
}
