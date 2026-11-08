// color.rs - Color/ANSI Library for JPX (rewritten from Python colorama)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let mut obj = HashMap::new();
    // ANSI color codes (equivalent to Python colorama)
    let colors = [
        ("red",     "\x1b[31m"),
        ("green",   "\x1b[32m"),
        ("yellow",  "\x1b[33m"),
        ("blue",    "\x1b[34m"),
        ("magenta", "\x1b[35m"),
        ("cyan",    "\x1b[36m"),
        ("white",   "\x1b[37m"),
        ("reset",   "\x1b[0m"),
    ];
    for (name, code) in &colors {
        let v = JpxValue::String(code.to_string());
        obj.insert(name.to_string(), v.clone());
        interp.env.insert(name.to_string(), v.clone());
        interp.builtins.insert(name.to_string(), v);
    }
    interp.env.insert("color".to_string(), JpxValue::Object(obj.clone()));
    interp.builtins.insert("color".to_string(), JpxValue::Object(obj));
    Ok(())
}
