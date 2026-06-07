// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub(crate) mod decompress;
pub(crate) mod http_client;

use andromeda_core::{Extension, ExtensionOp, HostData, OpsStorage, Rid};
use nova_vm::{
    ecmascript::{Agent, ArgumentsList, ExceptionType, JsResult, Value},
    engine::{Bindable, GcScope},
};

use crate::RuntimeMacroTask;
use decompress::{Decompressor, DecompressionResources};
use http_client::HttpClientResources;

#[derive(Default)]
pub struct FetchExt;

impl FetchExt {
    #[hotpath::measure]
    pub fn new_extension() -> Extension {
        Extension {
            name: "fetch",
            ops: vec![
                ExtensionOp::new(
                    "internal_set_response_decoder",
                    Self::internal_set_response_decoder,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_decompress_chunk",
                    Self::internal_decompress_chunk,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_decompress_finish",
                    Self::internal_decompress_finish,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_http_request",
                    http_client::internal_http_request,
                    4,
                    false,
                ),
                ExtensionOp::new(
                    "internal_http_response_read",
                    http_client::internal_http_response_read,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_http_response_close",
                    http_client::internal_http_response_close,
                    1,
                    false,
                ),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(DecompressionResources::new());
                storage.insert(HttpClientResources::new());
            })),
            files: vec![
                include_str!("./body/inner_body.ts"),
                include_str!("./body/extract.ts"),
                include_str!("./body/consume.ts"),
                include_str!("./body/body_mixin.ts"),
                include_str!("./headers.ts"),
                include_str!("./cors.ts"),
                include_str!("./response_filter.ts"),
                include_str!("./corp.ts"),
                include_str!("./mixed_content.ts"),
                include_str!("./sri.ts"),
                include_str!("./auth.ts"),
                include_str!("./cookies.ts"),
                include_str!("./request.ts"),
                include_str!("./response.ts"),
                include_str!("./fetch.ts"),
            ],
        }
    }

    fn internal_set_response_decoder<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let enc_binding = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let rid_str = rid_binding.as_str(agent).expect("non-UTF-8").to_string();
        let enc_str = enc_binding.as_str(agent).expect("non-UTF-8").to_string();

        let rid_val: u32 = match rid_str.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        ExceptionType::Error,
                        "Invalid RID",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };
        let rid = Rid::from_index(rid_val);

        let result_str = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let storage = host_data.storage.borrow();
            let resources: &DecompressionResources = storage.get().unwrap();
            match Decompressor::from_encoding(&enc_str) {
                Some(dec) => {
                    resources.insert(rid, dec);
                    "ok"
                }
                None => "passthrough",
            }
        };
        Ok(Value::from_string(agent, result_str.to_string(), gc.nogc()).unbind())
    }

    fn internal_decompress_chunk<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let rid_str = rid_binding.as_str(agent).expect("non-UTF-8").to_string();
        let bytes_value = args.get(1);

        let rid_val: u32 = match rid_str.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        ExceptionType::Error,
                        "Invalid RID",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };
        let rid = Rid::from_index(rid_val);

        let raw = match crate::ext::web::bytes::bytes_from_value(agent, bytes_value, gc.reborrow())
        {
            Ok(b) => b,
            Err(msg) => {
                return Err(agent
                    .throw_exception(
                        ExceptionType::Error,
                        format!("internal_decompress_chunk: {msg}"),
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let decoded: Result<Vec<u8>, String> = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let storage = host_data.storage.borrow();
            let resources: &DecompressionResources = storage.get().unwrap();
            if resources.has(rid) {
                resources.decode_chunk(rid, &raw)
            } else {
                Ok(raw)
            }
        };

        match decoded {
            Ok(bytes) => {
                use nova_vm::ecmascript::ArrayBuffer;
                let ab = ArrayBuffer::new(agent, bytes.len(), gc.nogc())
                    .expect("ArrayBuffer allocation failed");
                if !bytes.is_empty() {
                    ab.as_mut_slice(agent).copy_from_slice(&bytes);
                }
                Ok(Value::from(ab.unbind()).unbind())
            }
            Err(msg) => Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    format!("decode error: {msg}"),
                    gc.nogc(),
                )
                .unbind()),
        }
    }

    fn internal_decompress_finish<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let rid_binding = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let rid_str = rid_binding.as_str(agent).expect("non-UTF-8").to_string();
        let rid_val: u32 = match rid_str.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        ExceptionType::Error,
                        "Invalid RID",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };
        let rid = Rid::from_index(rid_val);

        let tail: Result<Vec<u8>, String> = {
            let host_data = agent.get_host_data();
            let host_data: &HostData<RuntimeMacroTask> = host_data.downcast_ref().unwrap();
            let storage = host_data.storage.borrow();
            let resources: &DecompressionResources = storage.get().unwrap();
            resources.finish_chunk(rid)
        };

        match tail {
            Ok(bytes) => {
                use nova_vm::ecmascript::ArrayBuffer;
                let ab = ArrayBuffer::new(agent, bytes.len(), gc.nogc())
                    .expect("ArrayBuffer allocation failed");
                if !bytes.is_empty() {
                    ab.as_mut_slice(agent).copy_from_slice(&bytes);
                }
                Ok(Value::from(ab.unbind()).unbind())
            }
            Err(msg) => Err(agent
                .throw_exception(
                    ExceptionType::Error,
                    format!("decode finish error: {msg}"),
                    gc.nogc(),
                )
                .unbind()),
        }
    }
}
