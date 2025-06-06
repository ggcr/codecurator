use thiserror::Error;

// To avoid the usage of Box<dyn Error>
// With this we catch all errors and implements the `from` trait
// to convert them to Error

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Network error {0}")]
    Http(#[from] reqwest::Error),

    #[error("Filesystem error {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation error: {message}")]
    Validation { message: String },
}

#[derive(Debug, Error)]
pub enum ExtractionError {
    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("YAML error")]
    Yaml(#[from] yaml_rust::ScanError),

    #[error("JSON lines error")]
    JsonlWriter(#[from] jsonl::WriteError),

    #[error("Tokenizer error")]
    Tokenizer { message: String },

    #[error("Zip error")]
    ZipErr(#[from] zip::result::ZipError),

    #[error("Validation error: {message}")]
    Validation { message: String },
}
