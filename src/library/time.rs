// time.rs - Time Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    #[allow(unused_imports)]
    use chrono::Local;

    let fns: &[(&str, JpxValue)] = &[
        ("time_now", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::String(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()))
        }))),
        ("time_today", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::String(chrono::Local::now().format("%Y-%m-%d").to_string()))
        }))),
        ("time_year", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::Int(chrono::Local::now().format("%Y").to_string().parse().unwrap_or(0)))
        }))),
        ("time_month", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::Int(chrono::Local::now().format("%m").to_string().parse().unwrap_or(0)))
        }))),
        ("time_day", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::Int(chrono::Local::now().format("%d").to_string().parse().unwrap_or(0)))
        }))),
        ("time_hour", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::Int(chrono::Local::now().format("%H").to_string().parse().unwrap_or(0)))
        }))),
        ("time_minute", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::Int(chrono::Local::now().format("%M").to_string().parse().unwrap_or(0)))
        }))),
        ("time_second", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::Int(chrono::Local::now().format("%S").to_string().parse().unwrap_or(0)))
        }))),
        ("time_timestamp", JpxValue::NativeFunction(Arc::new(|_| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64();
            Ok(JpxValue::Float(ts))
        }))),
        ("time_wait", JpxValue::NativeFunction(Arc::new(|args| {
            let secs = args.first().and_then(|v| v.to_f64()).unwrap_or(1.0);
            std::thread::sleep(std::time::Duration::from_secs_f64(secs));
            Ok(JpxValue::Bool(true))
        }))),
        ("time_sleep", JpxValue::NativeFunction(Arc::new(|args| {
            let secs = args.first().and_then(|v| v.to_f64()).unwrap_or(1.0);
            std::thread::sleep(std::time::Duration::from_secs_f64(secs));
            Ok(JpxValue::Bool(true))
        }))),
        ("time_format", JpxValue::NativeFunction(Arc::new(|args| {
            let fmt = args.first().map(|v| v.to_string()).unwrap_or_else(|| "%Y-%m-%d %H:%M:%S".to_string());
            Ok(JpxValue::String(chrono::Local::now().format(&fmt).to_string()))
        }))),
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    interp.env.insert("time".to_string(), JpxValue::Object(obj.clone()));
    interp.builtins.insert("time".to_string(), JpxValue::Object(obj));
    Ok(())
}
