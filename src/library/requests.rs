// requests.rs - HTTP Requests Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let fns: &[(&str, JpxValue)] = &[
        ("http_get", JpxValue::NativeFunction(Arc::new(|args| {
            let url = args.first().map(|v| v.to_string()).unwrap_or_default();
            match reqwest::blocking::get(&url) {
                Ok(resp) => {
                    let status = resp.status().as_u16() as i64;
                    let body = resp.text().unwrap_or_default();
                    let mut result = HashMap::new();
                    result.insert("status_code".to_string(), JpxValue::Int(status));
                    result.insert("body".to_string(), JpxValue::String(body.clone()));
                    result.insert("text".to_string(), JpxValue::String(body));
                    result.insert("ok".to_string(), JpxValue::Bool(status >= 200 && status < 300));
                    Ok(JpxValue::Object(result))
                }
                Err(e) => {
                    let mut result = HashMap::new();
                    result.insert("status_code".to_string(), JpxValue::Int(0));
                    result.insert("body".to_string(), JpxValue::String(e.to_string()));
                    result.insert("ok".to_string(), JpxValue::Bool(false));
                    Ok(JpxValue::Object(result))
                }
            }
        }))),
        ("http_post", JpxValue::NativeFunction(Arc::new(|args| {
            let url = args.first().map(|v| v.to_string()).unwrap_or_default();
            let data = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            let client = reqwest::blocking::Client::new();
            match client.post(&url).body(data).send() {
                Ok(resp) => {
                    let status = resp.status().as_u16() as i64;
                    let body = resp.text().unwrap_or_default();
                    let mut result = HashMap::new();
                    result.insert("status_code".to_string(), JpxValue::Int(status));
                    result.insert("body".to_string(), JpxValue::String(body.clone()));
                    result.insert("text".to_string(), JpxValue::String(body));
                    result.insert("ok".to_string(), JpxValue::Bool(status >= 200 && status < 300));
                    Ok(JpxValue::Object(result))
                }
                Err(e) => {
                    let mut result = HashMap::new();
                    result.insert("status_code".to_string(), JpxValue::Int(0));
                    result.insert("body".to_string(), JpxValue::String(e.to_string()));
                    result.insert("ok".to_string(), JpxValue::Bool(false));
                    Ok(JpxValue::Object(result))
                }
            }
        }))),
        ("http_fetch", JpxValue::NativeFunction(Arc::new(|args| {
            let url = args.first().map(|v| v.to_string()).unwrap_or_default();
            match reqwest::blocking::get(&url) {
                Ok(resp) => {
                    let status = resp.status().as_u16() as i64;
                    let body = resp.text().unwrap_or_default();
                    let mut result = HashMap::new();
                    result.insert("status".to_string(), JpxValue::Int(status));
                    result.insert("body".to_string(), JpxValue::String(body.clone()));
                    result.insert("text".to_string(), JpxValue::String(body));
                    result.insert("ok".to_string(), JpxValue::Bool(status >= 200 && status < 300));
                    Ok(JpxValue::Object(result))
                }
                Err(e) => {
                    let mut result = HashMap::new();
                    result.insert("status".to_string(), JpxValue::Int(0));
                    result.insert("body".to_string(), JpxValue::String(e.to_string()));
                    result.insert("ok".to_string(), JpxValue::Bool(false));
                    Ok(JpxValue::Object(result))
                }
            }
        }))),
        ("urlencode", JpxValue::NativeFunction(Arc::new(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::String(urlencoding::encode(&s).to_string()))
        }))),
        ("urldecode", JpxValue::NativeFunction(Arc::new(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::String(urlencoding::decode(&s).unwrap_or_default().to_string()))
        }))),
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    // Register as 'requests' and 'speedreq' objects
    interp.env.insert("requests".to_string(), JpxValue::Object(obj.clone()));
    interp.builtins.insert("requests".to_string(), JpxValue::Object(obj));
    Ok(())
}
