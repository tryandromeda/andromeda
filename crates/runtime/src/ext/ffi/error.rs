use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum FfiError {
    #[error("Failed to load library: {0}")]
    LibraryLoad(String),
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    #[error("Invalid function call: {0}")]
    InvalidCall(String),
    #[error("Type conversion error: {0}")]
    TypeConversion(String),
    #[error("Memory access error: {0}")]
    MemoryAccess(String),
}
