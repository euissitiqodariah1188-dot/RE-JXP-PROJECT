// notification.rs - Notification Library for JPX (rewritten from Python)
// Cross-platform: uses console fallback since win10toast is Windows-only
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let fns: &[(&str, JpxValue)] = &[
        ("notification_send", JpxValue::NativeFunction(Arc::new(|args| {
            let title = args.first().map(|v| v.to_string()).unwrap_or_else(|| "JPX Notification".to_string());
            let message = args.get(1).map(|v| v.to_string()).unwrap_or_else(|| "No message".to_string());
            // Try platform-native notification, fall back to console box
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("notify-send")
                    .arg(&title).arg(&message).spawn();
            }
            #[cfg(target_os = "macos")]
            {
                let script = format!("display notification \"{}\" with title \"{}\"", message, title);
                let _ = std::process::Command::new("osascript").arg("-e").arg(&script).spawn();
            }
            // Console fallback box
            let width = 60;
            println!("\n╔{}╗", "═".repeat(width));
            println!("║ {:^width$} ║", title, width = width - 2);
            println!("╠{}╣", "═".repeat(width));
            println!("║ {:width$} ║", message, width = width - 2);
            println!("╚{}╝", "═".repeat(width));

            let mut result = HashMap::new();
            result.insert("success".to_string(), JpxValue::Bool(true));
            result.insert("method".to_string(), JpxValue::String("console".to_string()));
            Ok(JpxValue::Object(result))
        }))),
        ("notification_info", JpxValue::NativeFunction(Arc::new(|args| {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_default();
            println!("\x1b[34mℹ [Info] {}\x1b[0m", msg);
            Ok(JpxValue::Bool(true))
        }))),
        ("notification_success", JpxValue::NativeFunction(Arc::new(|args| {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_default();
            println!("\x1b[32m✓ [Sukses] {}\x1b[0m", msg);
            Ok(JpxValue::Bool(true))
        }))),
        ("notification_warning", JpxValue::NativeFunction(Arc::new(|args| {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_default();
            println!("\x1b[33m⚠ [Peringatan] {}\x1b[0m", msg);
            Ok(JpxValue::Bool(true))
        }))),
        ("notification_error", JpxValue::NativeFunction(Arc::new(|args| {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_default();
            println!("\x1b[31m✗ [Error] {}\x1b[0m", msg);
            Ok(JpxValue::Bool(true))
        }))),
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    let mut notif_obj = HashMap::new();
    notif_obj.insert("send".to_string(), fns[0].1.clone());
    notif_obj.insert("info".to_string(), fns[1].1.clone());
    notif_obj.insert("success".to_string(), fns[2].1.clone());
    notif_obj.insert("warning".to_string(), fns[3].1.clone());
    notif_obj.insert("error".to_string(), fns[4].1.clone());
    interp.env.insert("notification".to_string(), JpxValue::Object(notif_obj.clone()));
    interp.builtins.insert("notification".to_string(), JpxValue::Object(notif_obj));
    Ok(())
}
