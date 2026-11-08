// scanner.rs - Interactive Input Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let fns: &[(&str, JpxValue)] = &[
        ("scanner_text", JpxValue::NativeFunction(Arc::new(|args| {
            let prompt = args.first().map(|v| v.to_string()).unwrap_or_default();
            print!("{}", prompt);
            use std::io::Write;
            std::io::stdout().flush().ok();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
            Ok(JpxValue::String(input.trim().to_string()))
        }))),
        ("scanner_number", JpxValue::NativeFunction(Arc::new(|args| {
            let prompt = args.first().map(|v| v.to_string()).unwrap_or_default();
            loop {
                print!("{}", prompt);
                use std::io::Write;
                std::io::stdout().flush().ok();
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).ok();
                let trimmed = input.trim();
                if let Ok(n) = trimmed.parse::<i64>() {
                    return Ok(JpxValue::Int(n));
                }
                if let Ok(f) = trimmed.parse::<f64>() {
                    return Ok(JpxValue::Float(f));
                }
                eprintln!("❌ Input angka!");
            }
        }))),
        ("scanner_yesno", JpxValue::NativeFunction(Arc::new(|args| {
            let prompt = args.first().map(|v| v.to_string()).unwrap_or_default();
            loop {
                print!("{} (y/n): ", prompt);
                use std::io::Write;
                std::io::stdout().flush().ok();
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).ok();
                let lower = input.trim().to_lowercase();
                if ["y", "yes", "ya"].contains(&lower.as_str()) { return Ok(JpxValue::Bool(true)); }
                if ["n", "no", "tidak"].contains(&lower.as_str()) { return Ok(JpxValue::Bool(false)); }
                eprintln!("❌ Jawab y/n");
            }
        }))),
        ("scanner_pause", JpxValue::NativeFunction(Arc::new(|_| {
            print!("Tekan Enter...");
            use std::io::Write;
            std::io::stdout().flush().ok();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
            Ok(JpxValue::Bool(true))
        }))),
        // input() as a shorthand
        ("input", JpxValue::NativeFunction(Arc::new(|args| {
            let prompt = args.first().map(|v| v.to_string()).unwrap_or_default();
            print!("{}", prompt);
            use std::io::Write;
            std::io::stdout().flush().ok();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
            Ok(JpxValue::String(input.trim().to_string()))
        }))),
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    interp.env.insert("scanner".to_string(), JpxValue::Object(obj.clone()));
    interp.builtins.insert("scanner".to_string(), JpxValue::Object(obj));
    Ok(())
}
