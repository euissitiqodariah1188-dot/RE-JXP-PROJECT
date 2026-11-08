// string.rs - String Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let fns: &[(&str, JpxValue)] = &[
        ("str_length", JpxValue::NativeFunction(Arc::new(|args| {
            Ok(JpxValue::Int(args.first().map(|v| v.to_string().len() as i64).unwrap_or(0)))
        }))),
        ("str_upper", JpxValue::NativeFunction(Arc::new(|args| {
            Ok(JpxValue::String(args.first().map(|v| v.to_string().to_uppercase()).unwrap_or_default()))
        }))),
        ("str_lower", JpxValue::NativeFunction(Arc::new(|args| {
            Ok(JpxValue::String(args.first().map(|v| v.to_string().to_lowercase()).unwrap_or_default()))
        }))),
        ("str_trim", JpxValue::NativeFunction(Arc::new(|args| {
            Ok(JpxValue::String(args.first().map(|v| v.to_string().trim().to_string()).unwrap_or_default()))
        }))),
        ("str_reverse", JpxValue::NativeFunction(Arc::new(|args| {
            Ok(JpxValue::String(args.first().map(|v| v.to_string().chars().rev().collect()).unwrap_or_default()))
        }))),
        ("str_contains", JpxValue::NativeFunction(Arc::new(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let sub = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::Bool(s.contains(&sub)))
        }))),
        ("str_replace", JpxValue::NativeFunction(Arc::new(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let old = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            let new = args.get(2).map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::String(s.replace(&old, &new)))
        }))),
        ("str_split", JpxValue::NativeFunction(Arc::new(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let delim = args.get(1).map(|v| v.to_string()).unwrap_or_else(|| " ".to_string());
            let parts: Vec<JpxValue> = s.split(&delim).map(|p| JpxValue::String(p.to_string())).collect();
            Ok(JpxValue::List(parts))
        }))),
        ("str_join", JpxValue::NativeFunction(Arc::new(|args| {
            let list = match args.first() { Some(JpxValue::List(v)) => v.clone(), _ => return Ok(JpxValue::String(String::new())) };
            let glue = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            let parts: Vec<String> = list.iter().map(|v| v.to_string()).collect();
            Ok(JpxValue::String(parts.join(&glue)))
        }))),
        ("str_starts_with", JpxValue::NativeFunction(Arc::new(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let prefix = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::Bool(s.starts_with(&prefix)))
        }))),
        ("str_ends_with", JpxValue::NativeFunction(Arc::new(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let suffix = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::Bool(s.ends_with(&suffix)))
        }))),
        ("str_is_email", JpxValue::NativeFunction(Arc::new(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let re = regex::Regex::new(r"^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$").unwrap();
            Ok(JpxValue::Bool(re.is_match(&s)))
        }))),
        ("str_b64encode", JpxValue::NativeFunction(Arc::new(|args| {
            use base64::Engine as _;
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::String(base64::engine::general_purpose::STANDARD.encode(s.as_bytes())))
        }))),
        ("str_b64decode", JpxValue::NativeFunction(Arc::new(|args| {
            use base64::Engine as _;
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            match base64::engine::general_purpose::STANDARD.decode(s.as_bytes()) {
                Ok(b) => Ok(JpxValue::String(String::from_utf8_lossy(&b).to_string())),
                Err(_) => Ok(JpxValue::String(String::new())),
            }
        }))),
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    interp.env.insert("string".to_string(), JpxValue::Object(obj.clone()));
    interp.builtins.insert("string".to_string(), JpxValue::Object(obj));
    Ok(())
}
