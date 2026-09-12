pub trait KasaDownloader {}

pub enum DownloaderStore<T: KasaDownloader> {
    Uninitialized,
    Initialized(T),
}
