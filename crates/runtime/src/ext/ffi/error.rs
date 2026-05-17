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

impl From<FfiError> for andromeda_core::RuntimeError {
    fn from(err: FfiError) -> Self {
        let (operation, library): (&'static str, Option<String>) = match &err {
            FfiError::LibraryLoad(lib) => ("library_load", Some(lib.clone())),
            FfiError::SymbolNotFound(_) => ("symbol_lookup", None),
            FfiError::InvalidCall(_) => ("function_call", None),
            FfiError::TypeConversion(_) => ("type_conversion", None),
            FfiError::MemoryAccess(_) => ("memory_access", None),
        };
        match library {
            Some(lib) => andromeda_core::RuntimeError::ffi_call_error_with_library(
                operation,
                lib,
                err.to_string(),
            ),
            None => andromeda_core::RuntimeError::ffi_call_error(operation, err.to_string()),
        }
    }
}
