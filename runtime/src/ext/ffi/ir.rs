use crate::ext::ffi::{
    core::{ForeignFunction, NativeType, NativeValue},
    error::FfiError,
};
use nova_vm::{
    ecmascript::{
        execution::Agent,
        types::{BigInt, Value},
    },
    engine::context::{Bindable, GcScope},
};
use std::os::raw::c_void;
use std::ptr;

pub fn ffi_parse_bool_arg(arg: Value) -> Result<NativeValue, FfiError> {
    match arg {
        Value::Boolean(value) => Ok(NativeValue { bool_value: value }),
        _ => Err(FfiError::TypeConversion("Expected boolean".to_string())),
    }
}

pub fn ffi_parse_u8_arg(
    agent: &mut Agent,
    mut gc: GcScope,
    arg: Value,
) -> Result<NativeValue, FfiError> {
    match arg.to_number(agent, gc.reborrow()) {
        Ok(num) => {
            let value = num.unbind().into_f64(agent) as i64;
            if value >= 0 && value <= u8::MAX as i64 {
                Ok(NativeValue {
                    u8_value: value as u8,
                })
            } else {
                Err(FfiError::TypeConversion(
                    "u8 value out of range".to_string(),
                ))
            }
        }
        Err(_) => Err(FfiError::TypeConversion(
            "Expected number for u8".to_string(),
        )),
    }
}

pub fn ffi_parse_i8_arg(
    agent: &mut Agent,
    mut gc: GcScope,
    arg: Value,
) -> Result<NativeValue, FfiError> {
    match arg.to_number(agent, gc.reborrow()) {
        Ok(num) => {
            let value = num.unbind().into_f64(agent) as i64;
            if value >= i8::MIN as i64 && value <= i8::MAX as i64 {
                Ok(NativeValue {
                    i8_value: value as i8,
                })
            } else {
                Err(FfiError::TypeConversion(
                    "i8 value out of range".to_string(),
                ))
            }
        }
        Err(_) => Err(FfiError::TypeConversion(
            "Expected number for i8".to_string(),
        )),
    }
}

pub fn ffi_parse_u16_arg(
    agent: &mut Agent,
    mut gc: GcScope,
    arg: Value,
) -> Result<NativeValue, FfiError> {
    match arg.to_number(agent, gc.reborrow()) {
        Ok(num) => {
            let value = num.unbind().into_f64(agent) as i64;
            if value >= 0 && value <= u16::MAX as i64 {
                Ok(NativeValue {
                    u16_value: value as u16,
                })
            } else {
                Err(FfiError::TypeConversion(
                    "u16 value out of range".to_string(),
                ))
            }
        }
        Err(_) => Err(FfiError::TypeConversion(
            "Expected number for u16".to_string(),
        )),
    }
}

pub fn ffi_parse_i16_arg(
    agent: &mut Agent,
    mut gc: GcScope,
    arg: Value,
) -> Result<NativeValue, FfiError> {
    match arg.to_number(agent, gc.reborrow()) {
        Ok(num) => {
            let value = num.unbind().into_f64(agent) as i64;
            if value >= i16::MIN as i64 && value <= i16::MAX as i64 {
                Ok(NativeValue {
                    i16_value: value as i16,
                })
            } else {
                Err(FfiError::TypeConversion(
                    "i16 value out of range".to_string(),
                ))
            }
        }
        Err(_) => Err(FfiError::TypeConversion(
            "Expected number for i16".to_string(),
        )),
    }
}

pub fn ffi_parse_u32_arg(
    agent: &mut Agent,
    mut gc: GcScope,
    arg: Value,
) -> Result<NativeValue, FfiError> {
    match arg.to_number(agent, gc.reborrow()) {
        Ok(num) => {
            let value = num.unbind().into_f64(agent) as i64;
            if value >= 0 && value <= u32::MAX as i64 {
                Ok(NativeValue {
                    u32_value: value as u32,
                })
            } else {
                Err(FfiError::TypeConversion(
                    "u32 value out of range".to_string(),
                ))
            }
        }
        Err(_) => Err(FfiError::TypeConversion(
            "Expected number for u32".to_string(),
        )),
    }
}

pub fn ffi_parse_i32_arg(
    agent: &mut Agent,
    mut gc: GcScope,
    arg: Value,
) -> Result<NativeValue, FfiError> {
    match arg.to_number(agent, gc.reborrow()) {
        Ok(num) => {
            let value = num.unbind().into_f64(agent) as i32;
            Ok(NativeValue { i32_value: value })
        }
        Err(_) => Err(FfiError::TypeConversion(
            "Expected number for i32".to_string(),
        )),
    }
}

pub fn ffi_parse_u64_arg(
    agent: &mut Agent,
    mut gc: GcScope,
    arg: Value,
) -> Result<NativeValue, FfiError> {
    match arg.to_number(agent, gc.reborrow()) {
        Ok(num) => {
            let value = num.unbind().into_f64(agent) as u64;
            Ok(NativeValue { u64_value: value })
        }
        Err(_) => Err(FfiError::TypeConversion(
            "Expected number for u64".to_string(),
        )),
    }
}

pub fn ffi_parse_i64_arg(
    agent: &mut Agent,
    mut gc: GcScope,
    arg: Value,
) -> Result<NativeValue, FfiError> {
    match arg.to_number(agent, gc.reborrow()) {
        Ok(num) => {
            let value = num.unbind().into_f64(agent) as i64;
            Ok(NativeValue { i64_value: value })
        }
        Err(_) => Err(FfiError::TypeConversion(
            "Expected number for i64".to_string(),
        )),
    }
}

pub fn ffi_parse_usize_arg(
    agent: &mut Agent,
    mut gc: GcScope,
    arg: Value,
) -> Result<NativeValue, FfiError> {
    match arg.to_number(agent, gc.reborrow()) {
        Ok(num) => {
            let value = num.unbind().into_f64(agent) as usize;
            Ok(NativeValue { usize_value: value })
        }
        Err(_) => Err(FfiError::TypeConversion(
            "Expected number for usize".to_string(),
        )),
    }
}

pub fn ffi_parse_isize_arg(
    agent: &mut Agent,
    mut gc: GcScope,
    arg: Value,
) -> Result<NativeValue, FfiError> {
    match arg.to_number(agent, gc.reborrow()) {
        Ok(num) => {
            let value = num.unbind().into_f64(agent) as isize;
            Ok(NativeValue { isize_value: value })
        }
        Err(_) => Err(FfiError::TypeConversion(
            "Expected number for isize".to_string(),
        )),
    }
}

pub fn ffi_parse_f32_arg(
    agent: &mut Agent,
    mut gc: GcScope,
    arg: Value,
) -> Result<NativeValue, FfiError> {
    match arg.to_number(agent, gc.reborrow()) {
        Ok(num) => Ok(NativeValue {
            f32_value: num.unbind().into_f64(agent) as f32,
        }),
        Err(_) => Err(FfiError::TypeConversion(
            "Expected number for f32".to_string(),
        )),
    }
}

pub fn ffi_parse_f64_arg(
    agent: &mut Agent,
    mut gc: GcScope,
    arg: Value,
) -> Result<NativeValue, FfiError> {
    match arg.to_number(agent, gc.reborrow()) {
        Ok(num) => Ok(NativeValue {
            f64_value: num.unbind().into_f64(agent),
        }),
        Err(_) => Err(FfiError::TypeConversion(
            "Expected number for f64".to_string(),
        )),
    }
}

pub fn ffi_parse_pointer_arg(arg: Value, _gc: GcScope) -> Result<NativeValue, FfiError> {
    match arg {
        Value::Null => Ok(NativeValue {
            pointer: ptr::null_mut(),
        }),
        Value::BigInt(bigint_value) => {
            let bigint_as_bigint: BigInt = bigint_value.into();
            let pointer_value = match bigint_as_bigint {
                BigInt::SmallBigInt(small) => small.into_i64() as usize,
                BigInt::BigInt(_heap_big) => {
                    // TODO: Implement proper large BigInt to pointer conversion
                    0usize
                }
            };
            Ok(NativeValue {
                pointer: pointer_value as *mut c_void,
            })
        }
        Value::SmallBigInt(small_bigint) => {
            let int_value = small_bigint.into_i64();
            Ok(NativeValue {
                pointer: int_value as usize as *mut c_void,
            })
        }
        _ => Err(FfiError::TypeConversion(
            "Expected null or BigInt for pointer".to_string(),
        )),
    }
}

pub fn parse_foreign_function<'gc>(
    _agent: &mut Agent,
    definition_value: Value,
    _gc: GcScope<'gc, '_>,
) -> Result<ForeignFunction, FfiError> {
    let Value::Object(_def_obj) = definition_value else {
        return Err(FfiError::TypeConversion(
            "Symbol definition must be an object".to_string(),
        ));
    };

    // TODO: Implement full JavaScript object property parsing
    let foreign_func = ForeignFunction {
        name: None,
        parameters: vec![NativeType::Pointer], // Default to pointer for most Windows APIs
        result: NativeType::I32,               // Most Windows APIs return HANDLE/DWORD/etc
        nonblocking: false,
        optional: false,
    };

    Ok(foreign_func)
}

// Simple boolean conversion helper
#[allow(dead_code)]
pub fn value_to_boolean(value: Value) -> bool {
    match value {
        Value::Boolean(b) => b,
        Value::Undefined | Value::Null => false,
        Value::Integer(i) => i.into_i64() != 0,
        Value::SmallF64(f) => !f.into_f64().is_nan() && f.into_f64() != 0.0,
        Value::Number(_) => true, // Simplified: assume non-zero numbers are true
        Value::String(_) | Value::SmallString(_) => true, // Simplified: assume non-empty strings are true
        _ => true,
    }
}

pub fn ffi_parse_buffer_arg(arg: Value) -> Result<NativeValue, FfiError> {
    // TODO: Handle ArrayBuffer/TypedArray
    match arg {
        Value::Null => Ok(NativeValue {
            pointer: ptr::null_mut(),
        }),
        _ => Err(FfiError::TypeConversion(
            "Expected ArrayBuffer or TypedArray for buffer".to_string(),
        )),
    }
}
