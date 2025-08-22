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
                ExtensionOp::new("internal_url_get_origin", Self::internal_get_origin, 1),
                ExtensionOp::new("internal_url_get_username", Self::internal_get_username, 1),
                ExtensionOp::new("internal_url_get_password", Self::internal_get_password, 1),
                ExtensionOp::new("internal_url_get_host", Self::internal_get_host, 1),
                ExtensionOp::new("internal_url_get_hostname", Self::internal_get_hostname, 1),
                ExtensionOp::new("internal_url_get_port", Self::internal_get_port, 1),
                ExtensionOp::new("internal_url_set_hostname", Self::internal_set_hostname, 2),
                ExtensionOp::new("internal_url_set_port", Self::internal_set_port, 2),
                ExtensionOp::new("internal_url_get_pathname", Self::internal_get_pathname, 1),
                ExtensionOp::new("internal_url_set_pathname", Self::internal_set_pathname, 2),
                ExtensionOp::new("internal_url_get_search", Self::internal_get_search, 1),
                ExtensionOp::new("internal_url_set_search", Self::internal_set_search, 2),
                ExtensionOp::new("internal_url_get_hash", Self::internal_get_hash, 1),
                ExtensionOp::new("internal_url_set_hash", Self::internal_set_hash, 2),
                ExtensionOp::new("internal_url_set_username", Self::internal_set_username, 2),
                ExtensionOp::new("internal_url_set_password", Self::internal_set_password, 2),
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
            Ok(url) => {
                Ok(Value::from_string(agent, format!("{}:", url.scheme()), gc.nogc()).unbind())
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_get_origin<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(url) => {
                // origin is scheme + '://' + host (including port if present)
                let scheme = url.scheme();
                if let Some(host) = url.host_str() {
                    let origin = if let Some(port) = url.port() {
                        format!("{scheme}://{host}:{port}")
                    } else {
                        format!("{scheme}://{host}")
                    };
                    Ok(Value::from_string(agent, origin, gc.nogc()).unbind())
                } else {
                    Ok(Value::from_string(agent, "".to_string(), gc.nogc()).unbind())
                }
            }
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

    fn internal_set_hostname<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url_str = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let new_host = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url_str.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(mut url) => {
                if url
                    .set_host(Some(
                        new_host.as_str(agent).expect("String is not valid UTF-8"),
                    ))
                    .is_err()
                {
                    return Ok(Value::from_string(agent, "".to_string(), gc.nogc()).unbind());
                }
                Ok(Value::from_string(agent, url.to_string(), gc.nogc()).unbind())
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_set_port<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url_str = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let new_port = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url_str.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(mut url) => {
                let s = new_port.as_str(agent).expect("String is not valid UTF-8");
                if s.is_empty() {
                    url.set_port(None).ok();
                } else if let Ok(p) = s.parse::<u16>() {
                    url.set_port(Some(p)).ok();
                }
                Ok(Value::from_string(agent, url.to_string(), gc.nogc()).unbind())
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_set_pathname<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url_str = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let new_path = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url_str.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(mut url) => {
                url.set_path(new_path.as_str(agent).expect("String is not valid UTF-8"));
                Ok(Value::from_string(agent, url.to_string(), gc.nogc()).unbind())
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_set_search<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url_str = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let new_search = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url_str.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(mut url) => {
                // new_search expected to include leading ? or be empty
                let s = new_search.as_str(agent).expect("String is not valid UTF-8");
                if s.is_empty() {
                    url.set_query(None);
                } else {
                    let trimmed = s.strip_prefix('?').unwrap_or(s);
                    url.set_query(Some(trimmed));
                }
                Ok(Value::from_string(agent, url.to_string(), gc.nogc()).unbind())
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_set_hash<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url_str = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let new_hash = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url_str.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(mut url) => {
                let s = new_hash.as_str(agent).expect("String is not valid UTF-8");
                let trimmed = s.strip_prefix('#').unwrap_or(s);
                if trimmed.is_empty() {
                    url.set_fragment(None);
                } else {
                    url.set_fragment(Some(trimmed));
                }
                Ok(Value::from_string(agent, url.to_string(), gc.nogc()).unbind())
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_set_username<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url_str = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let new_username = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url_str.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(mut url) => {
                url.set_username(
                    new_username
                        .as_str(agent)
                        .expect("String is not valid UTF-8"),
                )
                .ok();
                Ok(Value::from_string(agent, url.to_string(), gc.nogc()).unbind())
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }

    fn internal_set_password<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let url_str = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let new_password = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        match Url::parse(url_str.as_str(agent).expect("String is not valid UTF-8")) {
            Ok(mut url) => {
                url.set_password(Some(
                    new_password
                        .as_str(agent)
                        .expect("String is not valid UTF-8"),
                ))
                .ok();
                Ok(Value::from_string(agent, url.to_string(), gc.nogc()).unbind())
            }
            Err(e) => Ok(Value::from_string(agent, format!("Error: {e}"), gc.nogc()).unbind()),
        }
    }
}
