// env.rs - Environment Variables Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let fns: &[(&str, JpxValue)] = &[
        ("env_get", JpxValue::NativeFunction(Arc::new(|args| {
            let key = args.first().map(|v| v.to_string()).unwrap_or_default();
            let default = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::String(std::env::var(&key).unwrap_or(default)))
        }))),
        ("env_set", JpxValue::NativeFunction(Arc::new(|args| {
            let key = args.first().map(|v| v.to_string()).unwrap_or_default();
            let val = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            std::env::set_var(&key, &val);
            Ok(JpxValue::Bool(true))
        }))),
        ("env_has", JpxValue::NativeFunction(Arc::new(|args| {
            let key = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::Bool(std::env::var(&key).is_ok()))
        }))),
        ("env_delete", JpxValue::NativeFunction(Arc::new(|args| {
            let key = args.first().map(|v| v.to_string()).unwrap_or_default();
            std::env::remove_var(&key);
            Ok(JpxValue::Bool(true))
        }))),
        ("env_load", JpxValue::NativeFunction(Arc::new(|args| {
            let filename = args.first().map(|v| v.to_string()).unwrap_or_else(|| ".env".to_string());
            match std::fs::read_to_string(&filename) {
                Ok(content) => {
                    for line in content.lines() {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') || line.starts_with("//") { continue; }
                        if let Some(eq) = line.find('=') {
                            let key = line[..eq].trim();
                            let mut val = line[eq+1..].trim().to_string();
                            if (val.starts_with('"') && val.ends_with('"')) || (val.starts_with('\'') && val.ends_with('\'')) {
                                val = val[1..val.len()-1].to_string();
                            }
                            std::env::set_var(key, &val);
                        }
                    }
                    Ok(JpxValue::Bool(true))
                }
                Err(_) => Ok(JpxValue::Bool(false)),
            }
        }))),
        ("env_list", JpxValue::NativeFunction(Arc::new(|_| {
            let keys: Vec<JpxValue> = std::env::vars().map(|(k, _)| JpxValue::String(k)).collect();
            Ok(JpxValue::List(keys))
        }))),
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    interp.env.insert("env".to_string(), JpxValue::Object(obj.clone()));
    interp.builtins.insert("env".to_string(), JpxValue::Object(obj));
    Ok(())
}
