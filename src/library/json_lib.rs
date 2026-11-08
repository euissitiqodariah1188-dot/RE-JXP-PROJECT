// json_lib.rs - JSON Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

fn jpx_to_json(val: &JpxValue) -> serde_json::Value {
    match val {
        JpxValue::Null => serde_json::Value::Null,
        JpxValue::Bool(b) => serde_json::Value::Bool(*b),
        JpxValue::Int(n) => serde_json::json!(*n),
        JpxValue::Float(f) => serde_json::json!(*f),
        JpxValue::String(s) => serde_json::Value::String(s.clone()),
        JpxValue::List(v) => serde_json::Value::Array(v.iter().map(jpx_to_json).collect()),
        JpxValue::Object(m) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in m { obj.insert(k.clone(), jpx_to_json(v)); }
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::Null,
    }
}

fn json_to_jpx(val: serde_json::Value) -> JpxValue {
    match val {
        serde_json::Value::Null => JpxValue::Null,
        serde_json::Value::Bool(b) => JpxValue::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() { JpxValue::Int(i) }
            else { JpxValue::Float(n.as_f64().unwrap_or(0.0)) }
        }
        serde_json::Value::String(s) => JpxValue::String(s),
        serde_json::Value::Array(a) => JpxValue::List(a.into_iter().map(json_to_jpx).collect()),
        serde_json::Value::Object(m) => {
            JpxValue::Object(m.into_iter().map(|(k, v)| (k, json_to_jpx(v))).collect())
        }
    }
}

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let fns: &[(&str, JpxValue)] = &[
        ("json_encode", JpxValue::NativeFunction(Arc::new(|args| {
            if args.is_empty() { return Ok(JpxValue::String("null".to_string())); }
            let jv = jpx_to_json(args.first().unwrap_or(&JpxValue::Null));
            Ok(JpxValue::String(serde_json::to_string(&jv).unwrap_or("{}".to_string())))
        }))),
        ("json_decode", JpxValue::NativeFunction(Arc::new(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            match serde_json::from_str::<serde_json::Value>(&s) {
                Ok(v) => Ok(json_to_jpx(v)),
                Err(_) => Ok(JpxValue::Object(HashMap::new())),
            }
        }))),
        ("json_pretty", JpxValue::NativeFunction(Arc::new(|args| {
            if args.is_empty() { return Ok(JpxValue::String("{}".to_string())); }
            let jv = jpx_to_json(args.first().unwrap_or(&JpxValue::Null));
            Ok(JpxValue::String(serde_json::to_string_pretty(&jv).unwrap_or("{}".to_string())))
        }))),
        ("json_read", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            match std::fs::read_to_string(&path).and_then(|s| Ok(serde_json::from_str::<serde_json::Value>(&s).unwrap_or(serde_json::Value::Object(Default::default())))) {
                Ok(v) => Ok(json_to_jpx(v)),
                Err(_) => Ok(JpxValue::Object(HashMap::new())),
            }
        }))),
        ("json_write", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            if let Some(val) = args.get(1) {
                let jv = jpx_to_json(val);
                let s = serde_json::to_string_pretty(&jv).unwrap_or("{}".to_string());
                return match std::fs::write(&path, s) {
                    Ok(_) => Ok(JpxValue::Bool(true)),
                    Err(_) => Ok(JpxValue::Bool(false)),
                };
            }
            Ok(JpxValue::Bool(false))
        }))),
        ("json_keys", JpxValue::NativeFunction(Arc::new(|args| {
            if let Some(JpxValue::Object(m)) = args.first() {
                return Ok(JpxValue::List(m.keys().map(|k| JpxValue::String(k.clone())).collect()));
            }
            Ok(JpxValue::List(vec![]))
        }))),
        ("json_values", JpxValue::NativeFunction(Arc::new(|args| {
            if let Some(JpxValue::Object(m)) = args.first() {
                return Ok(JpxValue::List(m.values().cloned().collect()));
            }
            Ok(JpxValue::List(vec![]))
        }))),
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    interp.env.insert("json".to_string(), JpxValue::Object(obj.clone()));
    interp.builtins.insert("json".to_string(), JpxValue::Object(obj));
    Ok(())
}
