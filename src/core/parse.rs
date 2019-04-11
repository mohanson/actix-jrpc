use super::convention::{ErrorData, Request, JSONRPC_VERSION};
use json::JsonValue;



fn rpc_parse_method(recv_data: &json::object::Object) -> Option<String> {
    let method = recv_data.get("method");
    let method = match method {
        Some(some) => some,
        _ => return None,
    };
    let method = match method {
        JsonValue::Short(obj) => String::from(obj.as_str()),
        _ => return None,
    };
    Some(method)
}

fn rpc_parse_params(recv_data: &json::object::Object) -> Option<Vec<JsonValue>> {
    let params = recv_data.get("params");
    let params = match params {
        Some(some) => some,
        _ => return None,
    };
    let params = match params {
        JsonValue::Array(obj) => obj,
        _ => return None,
    };
    Some(params.clone())
}

pub fn parse(data: &[u8]) -> Result<Request, ErrorData> {
    let r = std::str::from_utf8(&data);
    let r = match r {
        Ok(ok) => ok,
        Err(e) => {
            let mut r = ErrorData::std(-32700);
            r.data = JsonValue::from(format!("{}", e));
            return Err(r);
        }
    };
    let recv = json::parse(r);
    let recv = match recv {
        Ok(ok) => ok,
        Err(e) => {
            let mut r = ErrorData::std(-32700);
            r.data = JsonValue::String(format!("{}", e));
            return Err(r);
        }
    };
    let recv = match recv {
        JsonValue::Object(obj) => obj,
        _ => {
            return Err(ErrorData::std(-32600));
        }
    };
    if recv.get("jsonrpc") != Some(&JsonValue::from(JSONRPC_VERSION)) {
        return Err(ErrorData::std(-32600));
    };
    let id = match recv.get("id") {
        Some(some) => some,
        None => return Err(ErrorData::std(-32600)),
    };
    let method = match rpc_parse_method(&recv) {
        Some(some) => some,
        None => {
            return Err(ErrorData::std(-32601));
        }
    };
    let params = match rpc_parse_params(&recv) {
        Some(some) => some,
        None => {
            return Err(ErrorData::std(-32602));
        }
    };
    Ok(Request {
        jsonrpc: String::from(JSONRPC_VERSION),
        method: method,
        params: params,
        id: id.clone(),
    })
}
