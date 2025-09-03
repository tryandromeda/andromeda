// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
mod core;
mod error;
mod ir;
use std::collections::HashMap;
use std::os::raw::c_void;
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use crate::ext::ffi::core::{
    CallbackDefinition, CallbackMap, DynamicLibrary, FfiCallback, FfiSymbol, ForeignFunction,
    Library, LibraryMap, NativeType, NativeValue,
};
use crate::ext::ffi::error::FfiError;
use crate::ext::ffi::ir::{
    ffi_parse_bool_arg, ffi_parse_buffer_arg, ffi_parse_f32_arg, ffi_parse_f64_arg,
    ffi_parse_i8_arg, ffi_parse_i16_arg, ffi_parse_i32_arg, ffi_parse_i64_arg, ffi_parse_isize_arg,
    ffi_parse_pointer_arg, ffi_parse_u8_arg, ffi_parse_u16_arg, ffi_parse_u32_arg,
    ffi_parse_u64_arg, ffi_parse_usize_arg, parse_foreign_function,
};
use andromeda_core::{Extension, ExtensionOp, HostData, OpsStorage};
use libffi::middle;
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::{BigInt, Value},
    },
    engine::context::{Bindable, GcScope},
};

static LIBRARY_ID_COUNTER: AtomicU32 = AtomicU32::new(1);
static CALLBACK_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

#[derive(Default)]
pub struct FfiExt;

impl FfiExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "ffi",
            ops: vec![
                ExtensionOp::new("ffi_dlopen", Self::ffi_dlopen, 2),
                ExtensionOp::new("ffi_dlopen_get_symbol", Self::ffi_dlopen_get_symbol, 3),
                ExtensionOp::new("ffi_call_symbol", Self::ffi_call_symbol, 3),
                ExtensionOp::new("ffi_dlclose", Self::ffi_dlclose, 1),
                ExtensionOp::new("ffi_create_callback", Self::ffi_create_callback, 2),
                ExtensionOp::new(
                    "ffi_get_callback_pointer",
                    Self::ffi_get_callback_pointer,
                    1,
                ),
                ExtensionOp::new("ffi_callback_close", Self::ffi_callback_close, 1),
                ExtensionOp::new("ffi_pointer_create", Self::ffi_pointer_create, 1),
                ExtensionOp::new("ffi_pointer_equals", Self::ffi_pointer_equals, 2),
                ExtensionOp::new("ffi_pointer_offset", Self::ffi_pointer_offset, 2),
                ExtensionOp::new("ffi_pointer_value", Self::ffi_pointer_value, 1),
                ExtensionOp::new("ffi_pointer_of", Self::ffi_pointer_of, 1),
                ExtensionOp::new("ffi_read_memory", Self::ffi_read_memory, 3),
                ExtensionOp::new("ffi_write_memory", Self::ffi_write_memory, 3),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(LibraryMap::new());
                storage.insert(CallbackMap::new());
            })),
            files: vec![include_str!("mod.ts")],
        }
    }

    /// Marshal a JavaScript value to a native value based on expected type
    #[allow(dead_code)]
    fn marshal_argument<'gc>(
        agent: &mut Agent,
        value: Value,
        native_type: &NativeType,
        mut gc: GcScope<'gc, '_>,
    ) -> Result<NativeValue, FfiError> {
        match native_type {
            NativeType::Void => Ok(NativeValue { void_value: () }),
            NativeType::Bool => ffi_parse_bool_arg(value),
            NativeType::U8 => ffi_parse_u8_arg(agent, gc.reborrow(), value),
            NativeType::I8 => ffi_parse_i8_arg(agent, gc.reborrow(), value),
            NativeType::U16 => ffi_parse_u16_arg(agent, gc.reborrow(), value),
            NativeType::I16 => ffi_parse_i16_arg(agent, gc.reborrow(), value),
            NativeType::U32 => ffi_parse_u32_arg(agent, gc.reborrow(), value),
            NativeType::I32 => ffi_parse_i32_arg(agent, gc.reborrow(), value),
            NativeType::U64 => ffi_parse_u64_arg(agent, gc.reborrow(), value),
            NativeType::I64 => ffi_parse_i64_arg(agent, gc.reborrow(), value),
            NativeType::USize => ffi_parse_usize_arg(agent, gc.reborrow(), value),
            NativeType::ISize => ffi_parse_isize_arg(agent, gc.reborrow(), value),
            NativeType::F32 => ffi_parse_f32_arg(agent, gc.reborrow(), value),
            NativeType::F64 => ffi_parse_f64_arg(agent, gc.reborrow(), value),
            NativeType::Pointer => ffi_parse_pointer_arg(value, gc.reborrow()),
            NativeType::Buffer => ffi_parse_buffer_arg(value),
            NativeType::Function(_) => {
                // TODO: Handle function pointers properly
                ffi_parse_pointer_arg(value, gc.reborrow())
            }
        }
    }

    /// Call a native function using libffi with proper argument marshalling
    fn call_native_function(
        fn_ptr: *const c_void,
        args: &[NativeValue],
        param_types: &[NativeType],
        result_type: &NativeType,
    ) -> Result<f64, String> {
        let arg_types: Vec<middle::Type> = param_types.iter().map(|t| t.to_ffi_type()).collect();

        let result_ffi_type = result_type.to_ffi_type();

        let cif = middle::Cif::new(arg_types, result_ffi_type);

        let mut ffi_args: Vec<middle::Arg> = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            match &param_types[i] {
                NativeType::Bool => ffi_args.push(middle::arg(unsafe { &arg.bool_value })),
                NativeType::U8 => ffi_args.push(middle::arg(unsafe { &arg.u8_value })),
                NativeType::I8 => ffi_args.push(middle::arg(unsafe { &arg.i8_value })),
                NativeType::U16 => ffi_args.push(middle::arg(unsafe { &arg.u16_value })),
                NativeType::I16 => ffi_args.push(middle::arg(unsafe { &arg.i16_value })),
                NativeType::U32 => ffi_args.push(middle::arg(unsafe { &arg.u32_value })),
                NativeType::I32 => ffi_args.push(middle::arg(unsafe { &arg.i32_value })),
                NativeType::U64 => ffi_args.push(middle::arg(unsafe { &arg.u64_value })),
                NativeType::I64 => ffi_args.push(middle::arg(unsafe { &arg.i64_value })),
                NativeType::USize => ffi_args.push(middle::arg(unsafe { &arg.usize_value })),
                NativeType::ISize => ffi_args.push(middle::arg(unsafe { &arg.isize_value })),
                NativeType::F32 => ffi_args.push(middle::arg(unsafe { &arg.f32_value })),
                NativeType::F64 => ffi_args.push(middle::arg(unsafe { &arg.f64_value })),
                NativeType::Pointer | NativeType::Buffer | NativeType::Function(_) => {
                    ffi_args.push(middle::arg(unsafe { &arg.pointer }))
                }
                NativeType::Void => {}
            }
        }

        // Convert the function pointer to CodePtr
        let code_ptr = middle::CodePtr(fn_ptr as *mut c_void);

        // Call the function and handle result based on result type
        let result_value = match result_type {
            NativeType::Void => {
                let _: () = unsafe { cif.call(code_ptr, &ffi_args) };
                0.0
            }
            NativeType::Bool => {
                let result: bool = unsafe { cif.call(code_ptr, &ffi_args) };
                if result { 1.0 } else { 0.0 }
            }
            NativeType::U8 => {
                let result: u8 = unsafe { cif.call(code_ptr, &ffi_args) };
                result as f64
            }
            NativeType::I8 => {
                let result: i8 = unsafe { cif.call(code_ptr, &ffi_args) };
                result as f64
            }
            NativeType::U16 => {
                let result: u16 = unsafe { cif.call(code_ptr, &ffi_args) };
                result as f64
            }
            NativeType::I16 => {
                let result: i16 = unsafe { cif.call(code_ptr, &ffi_args) };
                result as f64
            }
            NativeType::U32 => {
                let result: u32 = unsafe { cif.call(code_ptr, &ffi_args) };
                result as f64
            }
            NativeType::I32 => {
                let result: i32 = unsafe { cif.call(code_ptr, &ffi_args) };
                result as f64
            }
            NativeType::U64 => {
                let result: u64 = unsafe { cif.call(code_ptr, &ffi_args) };
                result as f64
            }
            NativeType::I64 => {
                let result: i64 = unsafe { cif.call(code_ptr, &ffi_args) };
                result as f64
            }
            NativeType::USize => {
                let result: usize = unsafe { cif.call(code_ptr, &ffi_args) };
                result as f64
            }
            NativeType::ISize => {
                let result: isize = unsafe { cif.call(code_ptr, &ffi_args) };
                result as f64
            }
            NativeType::F32 => {
                let result: f32 = unsafe { cif.call(code_ptr, &ffi_args) };
                result as f64
            }
            NativeType::F64 => unsafe { cif.call(code_ptr, &ffi_args) },
            NativeType::Pointer | NativeType::Buffer | NativeType::Function(_) => {
                let result: *const c_void = unsafe { cif.call(code_ptr, &ffi_args) };
                result as usize as f64
            }
        };

        Ok(result_value)
    }

    /// Unmarshal a native result back to a JavaScript value
    #[allow(dead_code)]
    fn unmarshal_result<'gc>(
        agent: &mut Agent,
        result: f64,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement proper result unmarshalling based on result type
        Ok(Value::from_f64(agent, result, gc.nogc()).unbind())
    }

    fn with_library<T, F>(agent: &mut Agent, lib_id: u32, operation: F) -> Result<T, Value>
    where
        F: FnOnce(&mut DynamicLibrary) -> Result<T, String>,
    {
        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        let mut storage = host_data.storage.borrow_mut();
        let libraries = storage.get_mut::<LibraryMap>().ok_or(Value::Undefined)?;
        let lib = libraries.get_mut(&lib_id).ok_or(Value::Undefined)?;

        operation(lib).map_err(|_| Value::Undefined)
    }

    pub fn ffi_dlopen<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let filename = match args.get(0) {
            Value::String(s) => s
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string(),
            Value::SmallString(s) => s.as_str().expect("String is not valid UTF-8").to_string(),
            _ => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Filename must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let symbols_obj = args.get(1);

        let library = match unsafe { Library::new(&filename) } {
            Ok(lib) => lib,
            Err(_e) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Failed to load dynamic library",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let lib_id = LIBRARY_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dynamic_lib = DynamicLibrary {
            library: Arc::new(Mutex::new(library)),
            id: lib_id,
            symbols: HashMap::new(),
        };

        if let Value::Object(_obj) = symbols_obj {
            // TODO: Implement full symbol definition parsing
            // This requires careful Nova VM GC scope management that will be completed
            // in the next phase to avoid the complex borrowing conflicts.
            // For now, we'll parse definitions lazily in get_symbol when they're requested
        }

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

        if host_data.storage.borrow().get::<LibraryMap>().is_none() {
            host_data.storage.borrow_mut().insert(LibraryMap::new());
        }
        if host_data.storage.borrow().get::<CallbackMap>().is_none() {
            host_data.storage.borrow_mut().insert(CallbackMap::new());
        }

        host_data
            .storage
            .borrow_mut()
            .get_mut::<LibraryMap>()
            .unwrap()
            .insert(lib_id, dynamic_lib);

        Ok(Value::from_f64(agent, lib_id as f64, gc.nogc()).unbind())
    }

    pub fn ffi_dlopen_get_symbol<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let lib_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Library ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let symbol_name = match args.get(1) {
            Value::String(s) => s
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string(),
            Value::SmallString(s) => s.as_str().expect("String is not valid UTF-8").to_string(),
            _ => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Symbol name must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let definition = args.get(2);
        let parsed_def = match parse_foreign_function(agent, definition, gc.reborrow()) {
            Ok(def) => def,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Failed to parse function definition",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        match Self::with_library(agent, lib_id, |lib| {
            let symbol_bytes = format!("{symbol_name}\0").into_bytes();
            let lib_lock = lib.library.lock().unwrap();
            match unsafe { lib_lock.get::<*const c_void>(&symbol_bytes) } {
                Ok(symbol) => {
                    let symbol_ptr = *symbol;
                    let ffi_symbol = FfiSymbol {
                        name: symbol_name.clone(),
                        definition: ForeignFunction {
                            name: Some(symbol_name.clone()),
                            parameters: parsed_def.parameters,
                            result: parsed_def.result,
                            nonblocking: parsed_def.nonblocking,
                            optional: parsed_def.optional,
                        },
                        pointer: symbol_ptr,
                    };
                    lib.symbols.insert(symbol_name, ffi_symbol);
                    Ok(symbol_ptr as usize)
                }
                Err(e) => Err(format!("{e}")),
            }
        }) {
            Ok(pointer) => Ok(Value::from_f64(agent, pointer as f64, gc.nogc()).unbind()),
            Err(_) => Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "Failed to get symbol",
                    gc.nogc(),
                )
                .unbind()),
        }
    }

    pub fn ffi_call_symbol<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let lib_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Library ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let symbol_name = match args.get(1) {
            Value::String(s) => s
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string(),
            Value::SmallString(s) => s.as_str().expect("String is not valid UTF-8").to_string(),
            _ => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Symbol name must be a string",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        // Parse parameters from args.get(2) array
        let params_array = args.get(2);

        // Get the library data we need without holding the borrow
        let (fn_ptr, param_types, result_type) = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();

            let mut storage = host_data.storage.borrow_mut();
            let libraries = match storage.get_mut::<LibraryMap>() {
                Some(libs) => libs,
                None => {
                    drop(storage); // Release borrow before using agent
                    return Err(agent
                        .throw_exception_with_static_message(
                            nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                            "No libraries available",
                            gc.nogc(),
                        )
                        .unbind());
                }
            };

            let lib = match libraries.get_mut(&lib_id) {
                Some(library) => library,
                None => {
                    drop(storage); // Release borrow before using agent
                    return Err(agent
                        .throw_exception_with_static_message(
                            nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                            "Library not found",
                            gc.nogc(),
                        )
                        .unbind());
                }
            };

            // Find the symbol in our symbol map
            let symbol = match lib.symbols.get(&symbol_name) {
                Some(sym) => sym,
                None => {
                    drop(storage); // Release borrow before using agent
                    return Err(agent
                        .throw_exception_with_static_message(
                            nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                            "Symbol not found",
                            gc.nogc(),
                        )
                        .unbind());
                }
            };

            // Copy the data we need
            (
                symbol.pointer,
                symbol.definition.parameters.clone(),
                symbol.definition.result.clone(),
            )
        };
        // TODO: implement better arg marshalling
        let mut native_args = Vec::new();

        match params_array {
            Value::Object(_params_obj) => {
                for param_type in &param_types {
                    let default_arg = match param_type {
                        NativeType::Bool => NativeValue { bool_value: false },
                        NativeType::U8 => NativeValue { u8_value: 0 },
                        NativeType::I8 => NativeValue { i8_value: 0 },
                        NativeType::U16 => NativeValue { u16_value: 0 },
                        NativeType::I16 => NativeValue { i16_value: 0 },
                        NativeType::U32 => NativeValue { u32_value: 0 },
                        NativeType::I32 => NativeValue { i32_value: 0 },
                        NativeType::U64 => NativeValue { u64_value: 0 },
                        NativeType::I64 => NativeValue { i64_value: 0 },
                        NativeType::USize => NativeValue { usize_value: 0 },
                        NativeType::ISize => NativeValue { isize_value: 0 },
                        NativeType::F32 => NativeValue { f32_value: 0.0 },
                        NativeType::F64 => NativeValue { f64_value: 0.0 },
                        NativeType::Pointer | NativeType::Buffer | NativeType::Function(_) => {
                            NativeValue {
                                pointer: ptr::null_mut(),
                            }
                        }
                        NativeType::Void => NativeValue { void_value: () },
                    };
                    native_args.push(default_arg);
                }

                // TODO: Implement full argument parsing in a future iteration
                // The complex borrowing rules make it challenging to safely access
                // object properties while maintaining all the required lifetime constraints
            }
            _ => {
                for param_type in &param_types {
                    let default_arg = match param_type {
                        NativeType::Bool => NativeValue { bool_value: false },
                        NativeType::U8 => NativeValue { u8_value: 0 },
                        NativeType::I8 => NativeValue { i8_value: 0 },
                        NativeType::U16 => NativeValue { u16_value: 0 },
                        NativeType::I16 => NativeValue { i16_value: 0 },
                        NativeType::U32 => NativeValue { u32_value: 0 },
                        NativeType::I32 => NativeValue { i32_value: 0 },
                        NativeType::U64 => NativeValue { u64_value: 0 },
                        NativeType::I64 => NativeValue { i64_value: 0 },
                        NativeType::USize => NativeValue { usize_value: 0 },
                        NativeType::ISize => NativeValue { isize_value: 0 },
                        NativeType::F32 => NativeValue { f32_value: 0.0 },
                        NativeType::F64 => NativeValue { f64_value: 0.0 },
                        NativeType::Pointer | NativeType::Buffer | NativeType::Function(_) => {
                            NativeValue {
                                pointer: ptr::null_mut(),
                            }
                        }
                        NativeType::Void => continue, // Skip void parameters
                    };
                    native_args.push(default_arg);
                }
            }
        }

        match Self::call_native_function(fn_ptr, &native_args, &param_types, &result_type) {
            Ok(result) => Ok(Value::from_f64(agent, result, gc.nogc()).unbind()),
            Err(_) => Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "Failed to call symbol",
                    gc.nogc(),
                )
                .unbind()),
        }
    }

    pub fn ffi_dlclose<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let lib_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Library ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        host_data
            .storage
            .borrow_mut()
            .get_mut::<LibraryMap>()
            .unwrap()
            .remove(&lib_id);

        Ok(Value::Undefined.unbind())
    }

    pub fn ffi_create_callback<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Parse callback definition and JS function
        let _definition = args.get(0);
        let _js_function = args.get(1);

        let callback_id = CALLBACK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        // Create a placeholder callback
        let callback = FfiCallback {
            id: callback_id,
            definition: CallbackDefinition {
                parameters: vec![],
                result: Box::new(NativeType::Void),
            },
            pointer: 0x87654321 as *const c_void,
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        host_data
            .storage
            .borrow_mut()
            .get_mut::<CallbackMap>()
            .unwrap()
            .insert(callback_id, callback);

        Ok(Value::from_f64(agent, callback_id as f64, _gc.nogc()).unbind())
    }

    pub fn ffi_get_callback_pointer<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let _callback_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                // Return a default pointer value instead of throwing an error
                let pointer_bigint = BigInt::from_i64(agent, 0x87654321).unbind();
                return Ok(pointer_bigint.into());
            }
        };

        // Temporarily return a hardcoded pointer value (0x87654321)
        // TODO: Implement proper callback pointer retrieval
        let pointer_bigint = BigInt::from_i64(agent, 0x87654321).unbind();
        Ok(pointer_bigint.into())
    }

    pub fn ffi_callback_close<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let callback_id = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u32,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Callback ID must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let host_data = agent.get_host_data();
        let host_data: &HostData<crate::RuntimeMacroTask> = host_data.downcast_ref().unwrap();
        host_data
            .storage
            .borrow_mut()
            .get_mut::<CallbackMap>()
            .unwrap()
            .remove(&callback_id);

        Ok(Value::Undefined.unbind())
    }

    pub fn ffi_pointer_create<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let pointer_value = match args.get(0).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as u64,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::TypeError,
                        "Pointer value must be a number",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        // Return the pointer as a BigInt
        let bigint = BigInt::from_u64(agent, pointer_value).unbind();
        Ok(bigint.into())
    }

    pub fn ffi_pointer_equals<'gc>(
        _agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let ptr1 = args.get(0);
        let ptr2 = args.get(1);

        // Compare pointer values - for now, use a simple implementation
        // TODO: Implement proper pointer comparison
        let equals = match (ptr1, ptr2) {
            (Value::BigInt(_), Value::BigInt(_)) => {
                // For now, assume different BigInts are different pointers
                // A proper implementation would compare the actual values
                false
            }
            (Value::Null, Value::Null) => true,
            _ => false,
        };

        Ok(Value::Boolean(equals).unbind())
    }

    pub fn ffi_pointer_offset<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let ptr = args.get(0);
        let offset = match args.get(1).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as i64,
            Err(_) => 0,
        };

        // Add offset to pointer
        match ptr {
            Value::BigInt(bi) => {
                // For now, return the original pointer since we can't easily do BigInt arithmetic
                // TODO: Implement proper pointer arithmetic
                Ok(bi.unbind().into())
            }
            Value::Null => {
                // Null pointer + offset = offset
                let result_bigint = BigInt::from_i64(agent, offset).unbind();
                Ok(result_bigint.into())
            }
            _ => {
                // Convert to bigint and add offset
                let base_value = 0i64; // Default to 0 for non-pointer values
                let result_value = base_value + offset;
                let result_bigint = BigInt::from_i64(agent, result_value).unbind();
                Ok(result_bigint.into())
            }
        }
    }

    pub fn ffi_pointer_value<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // Get the pointer from arguments - it should already be a BigInt
        let ptr = args.get(0);

        // If it's already a BigInt, return it directly
        if let Value::BigInt(_) = ptr {
            return Ok(ptr.unbind());
        }

        // If it's not a BigInt, it might be a number, convert it
        if let Ok(num) = ptr.to_number(agent, gc.reborrow()) {
            let value = num.unbind().into_f64(agent) as i64;
            return Ok(BigInt::from_i64(agent, value).unbind().into());
        }

        // Fallback to 0
        Ok(BigInt::from_i64(agent, 0).unbind().into())
    }

    pub fn ffi_pointer_of<'gc>(
        _agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Get pointer of ArrayBuffer/TypedArray
        let _buffer = _args.get(0);

        // For now, return null
        Ok(Value::Null.unbind())
    }

    pub fn ffi_read_memory<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let _ptr = args.get(0);
        let _offset = match args.get(1).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as usize,
            Err(_) => 0,
        };
        let size = match args.get(2).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as usize,
            Err(_) => 0,
        };
        // TODO: Implement safe memory reading with proper bounds checking
        let _buffer = vec![0u8; size];

        // TODO:
        // 1. Validate the pointer is non-null and valid
        // 2. Check memory permissions
        // 3. Copy from the actual memory location:
        // unsafe {
        //     let src_ptr = (ptr_value + offset) as *const u8;
        //     std::ptr::copy_nonoverlapping(src_ptr, buffer.as_mut_ptr(), size);
        // }

        // TODO: create ArrayBuffer
        Ok(Value::from_f64(agent, size as f64, gc.nogc()).unbind())
    }

    pub fn ffi_write_memory<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let _ptr = args.get(0);
        let _offset = match args.get(1).to_number(agent, gc.reborrow()) {
            Ok(num) => num.unbind().into_f64(agent) as usize,
            Err(_) => 0,
        };
        let _data = args.get(2);
        // TODO: Implement actual memory writing with proper validation
        // 1. Validating the pointer is non-null and writable
        // 2. Converting JavaScript data to bytes
        // 3. Writing to the memory location:
        // unsafe {
        //     let dst_ptr = (ptr_value + offset) as *mut u8;
        //     std::ptr::copy_nonoverlapping(data.as_ptr(), dst_ptr, data.len());
        // }

        Ok(Value::Undefined.unbind())
    }
}
