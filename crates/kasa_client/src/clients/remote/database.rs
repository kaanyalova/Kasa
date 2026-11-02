use sqlx::{Pool, Sqlite};

use crate::clients::database::KasaDatabase;
use crate::clients::remote::rest_client::RemoteClient;

impl KasaDatabase for RemoteDb {}

#[derive(Clone)]
pub struct RemoteDb {
    pub client: RemoteClient,
    pub thumbs_pool: Pool<Sqlite>,
}

impl RemoteDb {
    pub fn new(client: RemoteClient, thumbs_pool: Pool<Sqlite>) -> Self {
        Self {
            client,
            thumbs_pool,
        }
    }
}
