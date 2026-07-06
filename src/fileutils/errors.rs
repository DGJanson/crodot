#[allow(clippy::enum_variant_names)]
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("Could not find file {0}")]
    FileNotFoundError(String),
    #[error("Could not read file {0}")]
    FileNotReadError(String),
}
