use sqlx::{Pool, Sqlite};
use tokio::sync::Mutex;

use crate::clients::database::KasaDatabase;

impl KasaDatabase for LocalDb {}

#[derive(Clone)]
pub struct LocalDb {
    pub main_pool: Pool<Sqlite>,
    pub thumbs_pool: Pool<Sqlite>,
}
impl LocalDb {
    pub fn new(main_pool: Pool<Sqlite>, thumbs_pool: Pool<Sqlite>) -> Self {
        Self {
            main_pool,
            thumbs_pool,
        }
    }
}
