// random.rs - Random Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use rand::{Rng, seq::SliceRandom};

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let fns: &[(&str, JpxValue)] = &[
        ("random_float", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::Float(rand::thread_rng().gen::<f64>()))
        }))),
        ("randint", JpxValue::NativeFunction(Arc::new(|args| {
            let a = args.first().and_then(|v| v.to_i64()).unwrap_or(0);
            let b = args.get(1).and_then(|v| v.to_i64()).unwrap_or(1);
            Ok(JpxValue::Int(rand::thread_rng().gen_range(a..=b)))
        }))),
        ("uniform", JpxValue::NativeFunction(Arc::new(|args| {
            let a = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            let b = args.get(1).and_then(|v| v.to_f64()).unwrap_or(1.0);
            Ok(JpxValue::Float(rand::thread_rng().gen_range(a..b)))
        }))),
        ("random_choice", JpxValue::NativeFunction(Arc::new(|args| {
            if let Some(JpxValue::List(v)) = args.first() {
                if v.is_empty() { return Ok(JpxValue::Null); }
                let idx = rand::thread_rng().gen_range(0..v.len());
                return Ok(v[idx].clone());
            }
            Ok(JpxValue::Null)
        }))),
        ("shuffle_list", JpxValue::NativeFunction(Arc::new(|args| {
            if let Some(JpxValue::List(v)) = args.first() {
                let mut v = v.clone();
                v.shuffle(&mut rand::thread_rng());
                return Ok(JpxValue::List(v));
            }
            Ok(JpxValue::Null)
        }))),
        ("random_uuid", JpxValue::NativeFunction(Arc::new(|_| {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let uuid = format!("{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
                rng.gen::<u32>(), rng.gen::<u16>(),
                rng.gen::<u16>() & 0x0fff,
                (rng.gen::<u16>() & 0x3fff) | 0x8000,
                rng.gen::<u64>() & 0xffffffffffff);
            Ok(JpxValue::String(uuid))
        }))),
        ("random_color", JpxValue::NativeFunction(Arc::new(|_| {
            let mut rng = rand::thread_rng();
            Ok(JpxValue::String(format!("#{:02x}{:02x}{:02x}",
                rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>())))
        }))),
        ("random_string", JpxValue::NativeFunction(Arc::new(|args| {
            let len = args.first().and_then(|v| v.to_i64()).unwrap_or(10) as usize;
            const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
            let s: String = (0..len).map(|_| CHARS[rand::thread_rng().gen_range(0..CHARS.len())] as char).collect();
            Ok(JpxValue::String(s))
        }))),
        ("coin", JpxValue::NativeFunction(Arc::new(|_| {
            Ok(JpxValue::Bool(rand::thread_rng().gen_bool(0.5)))
        }))),
        ("dice", JpxValue::NativeFunction(Arc::new(|args| {
            let sides = args.first().and_then(|v| v.to_i64()).unwrap_or(6);
            Ok(JpxValue::Int(rand::thread_rng().gen_range(1..=sides)))
        }))),
        ("random_password", JpxValue::NativeFunction(Arc::new(|args| {
            let len = args.first().and_then(|v| v.to_i64()).unwrap_or(12) as usize;
            const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
            let s: String = (0..len).map(|_| CHARS[rand::thread_rng().gen_range(0..CHARS.len())] as char).collect();
            Ok(JpxValue::String(s))
        }))),
        ("random_sample", JpxValue::NativeFunction(Arc::new(|args| {
            let list = match args.first() { Some(JpxValue::List(v)) => v.clone(), _ => return Ok(JpxValue::List(vec![])) };
            let k = args.get(1).and_then(|v| v.to_i64()).unwrap_or(1) as usize;
            let mut indices: Vec<usize> = (0..list.len()).collect();
            indices.shuffle(&mut rand::thread_rng());
            let sampled: Vec<JpxValue> = indices.into_iter().take(k).map(|i| list[i].clone()).collect();
            Ok(JpxValue::List(sampled))
        }))),
    ];

    let mut obj = std::collections::HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    interp.env.insert("random".to_string(), JpxValue::Object(obj.clone()));
    interp.builtins.insert("random".to_string(), JpxValue::Object(obj));
    Ok(())
}
