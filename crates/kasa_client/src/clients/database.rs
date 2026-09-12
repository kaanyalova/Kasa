pub trait KasaDatabase {}

#[derive(Clone)]
pub enum DbStore<T: KasaDatabase> {
    WaitingForFrontend,
    Initialized(T),
    Uninitialized,
    Errored(String),
}
