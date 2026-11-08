// beuty.rs - Terminal Beautification Library for JPX (rewritten from Python rich)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let fns: &[(&str, JpxValue)] = &[
        ("beuty_print", JpxValue::NativeFunction(Arc::new(|args| {
            println!("{}", args.first().map(|v| v.to_string()).unwrap_or_default());
            Ok(JpxValue::Null)
        }))),
        ("beuty_success", JpxValue::NativeFunction(Arc::new(|args| {
            println!("\x1b[32m✓ {}\x1b[0m", args.first().map(|v| v.to_string()).unwrap_or_default());
            Ok(JpxValue::Null)
        }))),
        ("beuty_error", JpxValue::NativeFunction(Arc::new(|args| {
            println!("\x1b[31m✗ {}\x1b[0m", args.first().map(|v| v.to_string()).unwrap_or_default());
            Ok(JpxValue::Null)
        }))),
        ("beuty_warn", JpxValue::NativeFunction(Arc::new(|args| {
            println!("\x1b[33m⚠ {}\x1b[0m", args.first().map(|v| v.to_string()).unwrap_or_default());
            Ok(JpxValue::Null)
        }))),
        ("beuty_info", JpxValue::NativeFunction(Arc::new(|args| {
            println!("\x1b[34mℹ {}\x1b[0m", args.first().map(|v| v.to_string()).unwrap_or_default());
            Ok(JpxValue::Null)
        }))),
        ("beuty_debug", JpxValue::NativeFunction(Arc::new(|args| {
            println!("\x1b[35m🔍 {}\x1b[0m", args.first().map(|v| v.to_string()).unwrap_or_default());
            Ok(JpxValue::Null)
        }))),
        ("beuty_rule", JpxValue::NativeFunction(Arc::new(|args| {
            let title = args.first().map(|v| v.to_string()).unwrap_or_default();
            if title.is_empty() {
                println!("{}", "─".repeat(50));
            } else {
                println!("─── {} ───", title);
            }
            Ok(JpxValue::Null)
        }))),
        ("beuty_panel", JpxValue::NativeFunction(Arc::new(|args| {
            let text = args.first().map(|v| v.to_string()).unwrap_or_default();
            let title = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            println!("╔══{}══╗", "═".repeat(text.len().max(title.len())));
            if !title.is_empty() { println!("║ {} ║", title); }
            println!("║ {} ║", text);
            println!("╚══{}══╝", "═".repeat(text.len().max(title.len())));
            Ok(JpxValue::Null)
        }))),
        ("beuty_clear", JpxValue::NativeFunction(Arc::new(|_| {
            print!("\x1b[2J\x1b[H");
            Ok(JpxValue::Null)
        }))),
        ("beuty_table", JpxValue::NativeFunction(Arc::new(|args| {
            if let Some(JpxValue::List(headers)) = args.first() {
                let header_strs: Vec<String> = headers.iter().map(|v| v.to_string()).collect();
                println!("{}", header_strs.join(" | "));
                println!("{}", "-".repeat(50));
                if let Some(JpxValue::List(row_list)) = args.get(1) {
                    for row in row_list {
                        if let JpxValue::List(cells) = row {
                            println!("{}", cells.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" | "));
                        }
                    }
                }
            }
            Ok(JpxValue::Null)
        }))),
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    // Register method aliases on beuty object
    let mut beuty_obj = HashMap::new();
    beuty_obj.insert("print".to_string(), fns[0].1.clone());
    beuty_obj.insert("success".to_string(), fns[1].1.clone());
    beuty_obj.insert("error".to_string(), fns[2].1.clone());
    beuty_obj.insert("warn".to_string(), fns[3].1.clone());
    beuty_obj.insert("info".to_string(), fns[4].1.clone());
    beuty_obj.insert("debug".to_string(), fns[5].1.clone());
    beuty_obj.insert("rule".to_string(), fns[6].1.clone());
    beuty_obj.insert("panel".to_string(), fns[7].1.clone());
    beuty_obj.insert("clear".to_string(), fns[8].1.clone());
    beuty_obj.insert("table".to_string(), fns[9].1.clone());
    interp.env.insert("beuty".to_string(), JpxValue::Object(beuty_obj.clone()));
    interp.builtins.insert("beuty".to_string(), JpxValue::Object(beuty_obj));
    Ok(())
}
