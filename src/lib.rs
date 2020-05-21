pub mod downloader;
pub mod nom_parser;

pub type AsyncError = Box<dyn std::error::Error + Send + Sync + 'static>;
