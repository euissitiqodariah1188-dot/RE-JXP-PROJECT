// regex_lib.rs - Regex Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let fns: &[(&str, JpxValue)] = &[
        ("regex_match", JpxValue::NativeFunction(Arc::new(|args| {
            let pattern = args.first().map(|v| v.to_string()).unwrap_or_default();
            let text = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            match regex::Regex::new(&pattern) {
                Ok(re) => Ok(re.find(&text)
                    .map(|m| JpxValue::String(m.as_str().to_string()))
                    .unwrap_or(JpxValue::Null)),
                Err(e) => Ok(JpxValue::String(format!("<error: {}>", e))),
            }
        }))),
        ("regex_search", JpxValue::NativeFunction(Arc::new(|args| {
            let pattern = args.first().map(|v| v.to_string()).unwrap_or_default();
            let text = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            match regex::Regex::new(&pattern) {
                Ok(re) => Ok(re.find(&text)
                    .map(|m| JpxValue::String(m.as_str().to_string()))
                    .unwrap_or(JpxValue::Null)),
                Err(e) => Ok(JpxValue::String(format!("<error: {}>", e))),
            }
        }))),
        ("regex_findall", JpxValue::NativeFunction(Arc::new(|args| {
            let pattern = args.first().map(|v| v.to_string()).unwrap_or_default();
            let text = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            match regex::Regex::new(&pattern) {
                Ok(re) => {
                    let matches: Vec<JpxValue> = re.find_iter(&text)
                        .map(|m| JpxValue::String(m.as_str().to_string()))
                        .collect();
                    Ok(JpxValue::List(matches))
                }
                Err(e) => Ok(JpxValue::List(vec![JpxValue::String(format!("<error: {}>", e))])),
            }
        }))),
        ("regex_replace", JpxValue::NativeFunction(Arc::new(|args| {
            let pattern = args.first().map(|v| v.to_string()).unwrap_or_default();
            let repl = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            let text = args.get(2).map(|v| v.to_string()).unwrap_or_default();
            match regex::Regex::new(&pattern) {
                Ok(re) => Ok(JpxValue::String(re.replace_all(&text, repl.as_str()).to_string())),
                Err(e) => Ok(JpxValue::String(format!("<error: {}>", e))),
            }
        }))),
        ("regex_split", JpxValue::NativeFunction(Arc::new(|args| {
            let pattern = args.first().map(|v| v.to_string()).unwrap_or_default();
            let text = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            match regex::Regex::new(&pattern) {
                Ok(re) => {
                    let parts: Vec<JpxValue> = re.split(&text)
                        .map(|p| JpxValue::String(p.to_string()))
                        .collect();
                    Ok(JpxValue::List(parts))
                }
                Err(e) => Ok(JpxValue::List(vec![JpxValue::String(format!("<error: {}>", e))])),
            }
        }))),
        ("regex_escape", JpxValue::NativeFunction(Arc::new(|args| {
            let text = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::String(regex::escape(&text)))
        }))),
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    // Register as 'regex' object (JPX name: ReGx)
    let mut regex_obj = HashMap::new();
    regex_obj.insert("match".to_string(), fns[0].1.clone());
    regex_obj.insert("search".to_string(), fns[1].1.clone());
    regex_obj.insert("findall".to_string(), fns[2].1.clone());
    regex_obj.insert("replace".to_string(), fns[3].1.clone());
    regex_obj.insert("sub".to_string(), fns[3].1.clone());
    regex_obj.insert("split".to_string(), fns[4].1.clone());
    regex_obj.insert("escape".to_string(), fns[5].1.clone());
    interp.env.insert("regex".to_string(), JpxValue::Object(regex_obj.clone()));
    interp.builtins.insert("regex".to_string(), JpxValue::Object(regex_obj));
    Ok(())
}
