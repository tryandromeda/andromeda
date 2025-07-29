// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{Extension, ExtensionOp};
use nova_vm::engine::context::Bindable;

use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::GcScope,
};
use url::Url;

#[derive(Default)]
pub struct URLExt;

impl URLExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "url",
            ops: vec![
                ExtensionOp::new("internal_url_parse", Self::internal_parse, 2),
                ExtensionOp::new(
                    "internal_url_parse_no_base",
                    Self::internal_parse_no_base,
                    1,
                ),
                ExtensionOp::new("internal_url_get_protocol", Self::internal_get_protocol, 1),
                ExtensionOp::new("internal_url_get_username", Self::internal_get_username, 1),
                ExtensionOp::new("internal_url_get_password", Self::internal_get_password, 1),
                ExtensionOp::new("internal_url_get_host", Self::internal_get_host, 1),
                ExtensionOp::new("internal_url_get_hostname", Self::internal_get_hostname, 1),
                ExtensionOp::new("internal_url_get_port", Self::internal_get_port, 1),
                ExtensionOp::new("internal_url_get_pathname", Self::internal_get_pathname, 1),
                ExtensionOp::new("internal_url_get_search", Self::internal_get_search, 1),
                ExtensionOp::new("internal_url_get_hash", Self::internal_get_hash, 1),
            ],
            storage: None,
            files: vec![include_str!("./mod.ts")],
        }
    }

    fn internal_parse<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url = args.get(0).to_string(agent, gc.reborrow()).unbind()?;

        let base_href = args.get(1).to_string(agent, gc.reborrow()).unbind()?;

        let base_url = match Url::parse(base_href.as_str(agent).expect("String is not valid UTF-8"))
        {
            Ok(url) => url,
            Err(e) => {
                return Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind());
            }
        };

        let url = match base_url.join(url.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(url) => url,
            Err(e) => {
                return Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind());
            }
        };

        Ok(Value::from_string(agent, url.to_string(), gc.nogc()).unbind())
    }

    fn internal_get_protocol<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(url) => Ok(Value::from_string(agent, url.scheme().to_string(), gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_get_username<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(url) => {
                Ok(Value::from_string(agent, url.username().to_string(), gc.nogc()).unbind())
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_get_password<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(url) => {
                Ok(
                    Value::from_string(agent, url.password().unwrap_or("").to_string(), gc.nogc())
                        .unbind(),
                )
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_get_host<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(url) => {
                Ok(
                    Value::from_string(agent, url.host_str().unwrap_or("").to_string(), gc.nogc())
                        .unbind(),
                )
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_get_hostname<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(url) => {
                Ok(
                    Value::from_string(agent, url.domain().unwrap_or("").to_string(), gc.nogc())
                        .unbind(),
                )
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_get_port<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(url) => Ok(Value::from_string(
                agent,
                url.port().map(|p| p.to_string()).unwrap_or("".to_string()),
                gc.nogc(),
            )
            .unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_get_pathname<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(url) => Ok(Value::from_string(agent, url.path().to_string(), gc.nogc()).unbind()),
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_get_search<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(url) => {
                Ok(
                    Value::from_string(agent, url.query().unwrap_or("").to_string(), gc.nogc())
                        .unbind(),
                )
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_get_hash<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(url) => {
                Ok(
                    Value::from_string(agent, url.fragment().unwrap_or("").to_string(), gc.nogc())
                        .unbind(),
                )
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_parse_no_base<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url = args.get(0).to_string(agent, gc.reborrow()).unbind()?;

        let url = match Url::parse(url.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(url) => url,
            Err(e) => {
                return Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind());
            }
        };

        Ok(Value::from_string(agent, url.to_string(), gc.nogc()).unbind())
    }
}
