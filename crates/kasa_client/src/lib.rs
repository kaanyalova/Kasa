<<<<<<< HEAD
use std::sync::{Arc, OnceLock};

use jni::jni_mangle;
=======
use std::sync::Arc;

>>>>>>> 822088046bca79d7b319dc78a2c1c4c4ed8e8d1e
use kasa_core::config::global_config::GlobalConfig;

use crate::clients::{KasaClient, local::LocalKasaClient, remote::RemoteKasaClient};

uniffi::setup_scaffolding!();

pub mod clients;
pub mod errors;
pub mod events;

<<<<<<< HEAD
static VM: once_cell::sync::OnceCell<jni::JavaVM> = once_cell::sync::OnceCell::new();
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub(crate) fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .on_thread_start(|| {
                if let Some(vm) = VM.get() {
                    let _ = vm.attach_current_thread(|_env| -> jni::errors::Result<()> { Ok(()) });
                }
            })
            .build()
            .expect("failed to build tokio runtime")
    })
}

#[uniffi::export]
fn new_local(config: &GlobalConfig) -> Arc<dyn KasaClient> {
    let client = Arc::new(LocalKasaClient::wait_for_frontend());

    let client_for_init = client.clone();
    let config = config.clone();
    let _ = runtime().spawn(async move {
        client_for_init.initialize(&config).await;
    });

    client
}

#[uniffi::export]
fn new_remote(config: &GlobalConfig) -> Arc<dyn KasaClient> {
    let client = Arc::new(RemoteKasaClient::wait_for_frontend());

    let client_for_init = client.clone();
    let config = config.clone();
    let _ = runtime().spawn(async move {
        client_for_init.initialize(&config).await;
    });

    client
}
#[jni_mangle("com.kaanb.kasa_android.jni.JniInit")]
pub extern "system" fn java_init<'local>(
    mut unowned_env: jni::EnvUnowned<'local>,
    _class: jni::objects::JClass<'local>,
    app: jni::objects::JObject<'local>,
) {
    let _ = unowned_env.with_env(|env| -> jni::errors::Result<()> {
        let vm = env.get_java_vm()?;
        let _ = VM.set(vm);
        Ok(())
    });
=======
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
>>>>>>> 822088046bca79d7b319dc78a2c1c4c4ed8e8d1e
}
