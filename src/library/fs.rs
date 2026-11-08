// fs.rs - File System Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let fns: &[(&str, JpxValue)] = &[
        ("fs_write", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            let content = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            match std::fs::write(&path, &content) {
                Ok(_) => Ok(JpxValue::Bool(true)),
                Err(e) => { eprintln!("Error writing file: {}", e); Ok(JpxValue::Bool(false)) }
            }
        }))),
        ("fs_read", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            match std::fs::read_to_string(&path) {
                Ok(s) => Ok(JpxValue::String(s)),
                Err(e) => { eprintln!("Error reading file: {}", e); Ok(JpxValue::Null) }
            }
        }))),
        ("fs_append", JpxValue::NativeFunction(Arc::new(|args| {
            use std::io::Write;
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            let content = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            match std::fs::OpenOptions::new().append(true).create(true).open(&path) {
                Ok(mut f) => { let _ = writeln!(f, "{}", content); Ok(JpxValue::Bool(true)) }
                Err(e) => { eprintln!("Error appending to file: {}", e); Ok(JpxValue::Bool(false)) }
            }
        }))),
        ("fs_exists", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::Bool(std::path::Path::new(&path).exists()))
        }))),
        ("fs_delete", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            match std::fs::remove_file(&path) {
                Ok(_) => Ok(JpxValue::Bool(true)),
                Err(e) => { eprintln!("Error deleting file: {}", e); Ok(JpxValue::Bool(false)) }
            }
        }))),
        ("fs_mkdir", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            match std::fs::create_dir_all(&path) {
                Ok(_) => Ok(JpxValue::Bool(true)),
                Err(e) => { eprintln!("Error creating directory: {}", e); Ok(JpxValue::Bool(false)) }
            }
        }))),
        ("fs_listdir", JpxValue::NativeFunction(Arc::new(|args| {
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
        ("fs_is_file", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::Bool(std::path::Path::new(&path).is_file()))
        }))),
        ("fs_is_dir", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::Bool(std::path::Path::new(&path).is_dir()))
        }))),
        ("fs_getcwd", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::String(std::env::current_dir().unwrap_or_default().to_string_lossy().to_string()))
        }))),
        ("fs_basename", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            let p = std::path::Path::new(&path);
            Ok(JpxValue::String(p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()))
        }))),
        ("fs_dirname", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            let p = std::path::Path::new(&path);
            Ok(JpxValue::String(p.parent().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()))
        }))),
        ("fs_abspath", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::String(std::fs::canonicalize(&path).unwrap_or_default().to_string_lossy().to_string()))
        }))),
        ("fs_size", JpxValue::NativeFunction(Arc::new(|args| {
            let path = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::Int(std::fs::metadata(&path).map(|m| m.len() as i64).unwrap_or(0)))
        }))),
        ("fs_copy", JpxValue::NativeFunction(Arc::new(|args| {
            let src = args.first().map(|v| v.to_string()).unwrap_or_default();
            let dst = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            match std::fs::copy(&src, &dst) {
                Ok(_) => Ok(JpxValue::Bool(true)),
                Err(e) => { eprintln!("Error copying file: {}", e); Ok(JpxValue::Bool(false)) }
            }
        }))),
        ("fs_rename", JpxValue::NativeFunction(Arc::new(|args| {
            let old = args.first().map(|v| v.to_string()).unwrap_or_default();
            let new = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            match std::fs::rename(&old, &new) {
                Ok(_) => Ok(JpxValue::Bool(true)),
                Err(e) => { eprintln!("Error renaming file: {}", e); Ok(JpxValue::Bool(false)) }
            }
        }))),
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    interp.env.insert("fs".to_string(), JpxValue::Object(obj.clone()));
    interp.builtins.insert("fs".to_string(), JpxValue::Object(obj));
    Ok(())
}
