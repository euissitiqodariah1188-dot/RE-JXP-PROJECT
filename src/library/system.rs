// system.rs - System Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let fns: &[(&str, JpxValue)] = &[
        ("system_run", JpxValue::NativeFunction(Arc::new(|args| {
            let cmd = args.first().map(|v| v.to_string()).unwrap_or_default();
            let status = std::process::Command::new("sh").arg("-c").arg(&cmd).status();
            Ok(JpxValue::Int(status.map(|s| s.code().unwrap_or(0) as i64).unwrap_or(-1)))
        }))),
        ("system_output", JpxValue::NativeFunction(Arc::new(|args| {
            let cmd = args.first().map(|v| v.to_string()).unwrap_or_default();
            let output = std::process::Command::new("sh").arg("-c").arg(&cmd).output();
            match output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                    Ok(JpxValue::String(format!("{}{}", stdout, stderr)))
                }
                Err(e) => Ok(JpxValue::String(e.to_string())),
            }
        }))),
        ("system_pwd", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::String(std::env::current_dir().unwrap_or_default().to_string_lossy().to_string()))
        }))),
        ("system_ls", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_else(|| ".".to_string());
            match std::fs::read_dir(&path) {
                Ok(entries) => {
                    let items: Vec<JpxValue> = entries.flatten()
                        .map(|e| JpxValue::String(e.file_name().to_string_lossy().to_string()))
                        .collect();
                    Ok(JpxValue::List(items))
                }
                Err(_) => Ok(JpxValue::List(vec![])),
            }
        }))),
        ("system_cd", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            match std::env::set_current_dir(&path) {
                Ok(_) => Ok(JpxValue::String(std::env::current_dir().unwrap_or_default().to_string_lossy().to_string())),
                Err(e) => Ok(JpxValue::String(e.to_string())),
            }
        }))),
        ("system_exists", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::Bool(std::path::Path::new(&path).exists()))
        }))),
        ("system_platform", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::String(std::env::consts::OS.to_string()))
        }))),
        ("system_cpu_count", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::Int(num_cpus() as i64))
        }))),
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    interp.env.insert("system".to_string(), JpxValue::Object(obj.clone()));
    interp.builtins.insert("system".to_string(), JpxValue::Object(obj));
    Ok(())
}

fn num_cpus() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
}
