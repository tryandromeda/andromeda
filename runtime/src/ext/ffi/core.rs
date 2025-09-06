use crate::ext::ffi::error::FfiError;
use libffi::middle;
use libloading::{Library as LibloadingLibrary, Symbol};
use std::collections::HashMap;
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum NativeType {
    Void,
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    USize,
    ISize,
    F32,
    F64,
    Pointer,
    Buffer,
    Function(Box<CallbackDefinition>),
    Struct(Box<StructDefinition>),
}

impl NativeType {
    #[allow(dead_code)]
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "void" => Some(NativeType::Void),
            "bool" => Some(NativeType::Bool),
            "u8" => Some(NativeType::U8),
            "i8" => Some(NativeType::I8),
            "u16" => Some(NativeType::U16),
            "i16" => Some(NativeType::I16),
            "u32" => Some(NativeType::U32),
            "i32" => Some(NativeType::I32),
            "u64" => Some(NativeType::U64),
            "i64" => Some(NativeType::I64),
            "usize" => Some(NativeType::USize),
            "isize" => Some(NativeType::ISize),
            "f32" => Some(NativeType::F32),
            "f64" => Some(NativeType::F64),
            "pointer" => Some(NativeType::Pointer),
            "buffer" => Some(NativeType::Buffer),
            _ => None,
        }
    }

    pub fn to_ffi_type(&self) -> middle::Type {
        match self {
            NativeType::Void => middle::Type::void(),
            NativeType::Bool => middle::Type::u8(),
            NativeType::U8 => middle::Type::u8(),
            NativeType::I8 => middle::Type::i8(),
            NativeType::U16 => middle::Type::u16(),
            NativeType::I16 => middle::Type::i16(),
            NativeType::U32 => middle::Type::u32(),
            NativeType::I32 => middle::Type::i32(),
            NativeType::U64 => middle::Type::u64(),
            NativeType::I64 => middle::Type::i64(),
            NativeType::USize => middle::Type::usize(),
            NativeType::ISize => middle::Type::isize(),
            NativeType::F32 => middle::Type::f32(),
            NativeType::F64 => middle::Type::f64(),
            NativeType::Pointer => middle::Type::pointer(),
            NativeType::Buffer => middle::Type::pointer(),
            NativeType::Function(_) => middle::Type::pointer(),
            NativeType::Struct(_) => middle::Type::pointer(),
        }
    }
}

pub type LibraryMap = HashMap<u32, DynamicLibrary>;
pub type CallbackMap = HashMap<u32, FfiCallback>;

/// Intermediate format for easy translation from NativeType + Nova VM value
#[repr(C)]
pub union NativeValue {
    pub void_value: (),
    pub bool_value: bool,
    pub u8_value: u8,
    pub i8_value: i8,
    pub u16_value: u16,
    pub i16_value: i16,
    pub u32_value: u32,
    pub i32_value: i32,
    pub u64_value: u64,
    pub i64_value: i64,
    pub usize_value: usize,
    pub isize_value: isize,
    pub f32_value: f32,
    pub f64_value: f64,
    pub pointer: *mut c_void,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DynamicLibrary {
    pub library: Arc<Mutex<Library>>,
    pub id: u32,
    pub symbols: HashMap<String, FfiSymbol>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct FfiCallback {
    pub id: u32,
    pub definition: CallbackDefinition,
    pub pointer: *const c_void,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CallbackDefinition {
    pub parameters: Vec<NativeType>,
    pub result: Box<NativeType>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StructDefinition {
    pub fields: Vec<NativeType>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct FfiSymbol {
    pub name: String,
    pub definition: ForeignFunction,
    pub pointer: *const c_void,
}

#[allow(clippy::non_send_fields_in_send_ty)]
// SAFETY: unsafe trait must have unsafe implementation
unsafe impl Send for FfiSymbol {}
// SAFETY: unsafe trait must have unsafe implementation
unsafe impl Sync for FfiSymbol {}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ForeignFunction {
    pub name: Option<String>,
    pub parameters: Vec<NativeType>,
    pub result: NativeType,
    pub nonblocking: bool,
    pub optional: bool,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Library {
    pub inner: LibloadingLibrary,
}

impl Library {
    /// Create a new library wrapper by loading a dynamic library from the given path.
    ///
    /// # Safety
    ///
    /// Loading a dynamic library can execute arbitrary code during library initialization.
    /// The caller must ensure that the library at the given path is trusted and that
    /// loading it will not cause undefined behavior or security vulnerabilities.
    pub unsafe fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, FfiError> {
        unsafe {
            LibloadingLibrary::new(path.as_ref())
                .map(|inner| Library { inner })
                .map_err(|e| FfiError::LibraryLoad(format!("{e}")))
        }
    }

    /// Get a symbol from the loaded library.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - The symbol name is valid and null-terminated
    /// - The symbol exists in the library and has the expected type T
    /// - The returned symbol will only be used in a way that's compatible with its actual type
    /// - The library remains loaded for the lifetime of the returned symbol
    pub unsafe fn get<T>(&self, symbol: &[u8]) -> Result<Symbol<'_, T>, FfiError> {
        unsafe {
            self.inner
                .get(symbol)
                .map_err(|e| FfiError::SymbolNotFound(format!("{e}")))
        }
    }
}
