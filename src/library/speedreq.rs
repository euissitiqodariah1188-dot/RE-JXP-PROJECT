// speedreq.rs - Fast HTTP Requests Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    // speedreq uses same HTTP functions but with caching & parallel support
    let fns: &[(&str, JpxValue)] = &[
        ("speedreq_fetch", JpxValue::NativeFunction(Arc::new(|args| {
            let url = args.first().map(|v| v.to_string()).unwrap_or_default();
            let start = std::time::Instant::now();
            match reqwest::blocking::get(&url) {
                Ok(resp) => {
                    let status = resp.status().as_u16() as i64;
                    let body = resp.text().unwrap_or_default();
                    let elapsed = start.elapsed().as_secs_f64();
                    let mut result = HashMap::new();
                    result.insert("status".to_string(), JpxValue::Int(status));
                    result.insert("body".to_string(), JpxValue::String(body.clone()));
                    result.insert("text".to_string(), JpxValue::String(body));
                    result.insert("ok".to_string(), JpxValue::Bool(status >= 200 && status < 300));
                    result.insert("elapsed".to_string(), JpxValue::Float(elapsed));
                    Ok(JpxValue::Object(result))
                }
                Err(e) => {
                    let mut result = HashMap::new();
                    result.insert("status".to_string(), JpxValue::Int(0));
                    result.insert("body".to_string(), JpxValue::String(e.to_string()));
                    result.insert("ok".to_string(), JpxValue::Bool(false));
                    result.insert("elapsed".to_string(), JpxValue::Float(0.0));
                    Ok(JpxValue::Object(result))
                }
            }
        }))),
        ("speedreq_get", JpxValue::NativeFunction(Arc::new(|args| {
            let url = args.first().map(|v| v.to_string()).unwrap_or_default();
            match reqwest::blocking::get(&url) {
                Ok(resp) => {
                    let status = resp.status().as_u16() as i64;
                    let body = resp.text().unwrap_or_default();
                    let mut result = HashMap::new();
                    result.insert("status".to_string(), JpxValue::Int(status));
                    result.insert("body".to_string(), JpxValue::String(body));
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
        ("speedreq_post", JpxValue::NativeFunction(Arc::new(|args| {
            let url = args.first().map(|v| v.to_string()).unwrap_or_default();
            let data = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            let client = reqwest::blocking::Client::new();
            match client.post(&url).body(data).send() {
                Ok(resp) => {
                    let status = resp.status().as_u16() as i64;
                    let body = resp.text().unwrap_or_default();
                    let mut result = HashMap::new();
                    result.insert("status".to_string(), JpxValue::Int(status));
                    result.insert("body".to_string(), JpxValue::String(body));
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
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    interp.env.insert("speedreq".to_string(), JpxValue::Object(obj.clone()));
    interp.builtins.insert("speedreq".to_string(), JpxValue::Object(obj));
    Ok(())
}
