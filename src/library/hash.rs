// hash.rs - Hash/Encoding Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let fns: &[(&str, JpxValue)] = &[
        ("base64_encode", JpxValue::NativeFunction(Arc::new(|args| {
            use base64::Engine as _;
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::String(base64::engine::general_purpose::STANDARD.encode(s.as_bytes())))
        }))),
        ("base64_decode", JpxValue::NativeFunction(Arc::new(|args| {
            use base64::Engine as _;
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            match base64::engine::general_purpose::STANDARD.decode(s.as_bytes()) {
                Ok(b) => Ok(JpxValue::String(String::from_utf8_lossy(&b).to_string())),
                Err(e) => Ok(JpxValue::String(format!("<error: {}>", e))),
            }
        }))),
        ("base64url_encode", JpxValue::NativeFunction(Arc::new(|args| {
            use base64::Engine as _;
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::String(base64::engine::general_purpose::URL_SAFE.encode(s.as_bytes())))
        }))),
        ("base16_encode", JpxValue::NativeFunction(Arc::new(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::String(hex::encode(s.as_bytes())))
        }))),
        ("base16_decode", JpxValue::NativeFunction(Arc::new(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            match hex::decode(&s) {
                Ok(b) => Ok(JpxValue::String(String::from_utf8_lossy(&b).to_string())),
                Err(e) => Ok(JpxValue::String(format!("<error: {}>", e))),
            }
        }))),
        ("md5", JpxValue::NativeFunction(Arc::new(|args| {
            use md5::{Md5, Digest};
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let mut h = Md5::new();
            h.update(s.as_bytes());
            Ok(JpxValue::String(format!("{:x}", h.finalize())))
        }))),
        ("sha1", JpxValue::NativeFunction(Arc::new(|args| {
            use sha1::{Sha1, Digest};
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let mut h = Sha1::new();
            h.update(s.as_bytes());
            Ok(JpxValue::String(format!("{:x}", h.finalize())))
        }))),
        ("sha256", JpxValue::NativeFunction(Arc::new(|args| {
            use sha2::{Sha256, Digest};
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let mut h = Sha256::new();
            h.update(s.as_bytes());
            Ok(JpxValue::String(format!("{:x}", h.finalize())))
        }))),
        ("sha384", JpxValue::NativeFunction(Arc::new(|args| {
            use sha2::{Sha384, Digest};
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let mut h = Sha384::new();
            h.update(s.as_bytes());
            Ok(JpxValue::String(format!("{:x}", h.finalize())))
        }))),
        ("sha512", JpxValue::NativeFunction(Arc::new(|args| {
            use sha2::{Sha512, Digest};
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let mut h = Sha512::new();
            h.update(s.as_bytes());
            Ok(JpxValue::String(format!("{:x}", h.finalize())))
        }))),
        ("sha3_256", JpxValue::NativeFunction(Arc::new(|args| {
            use sha3::{Sha3_256, Digest};
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let mut h = Sha3_256::new();
            h.update(s.as_bytes());
            Ok(JpxValue::String(format!("{:x}", h.finalize())))
        }))),
        ("blake2b", JpxValue::NativeFunction(Arc::new(|args| {
            use blake2::{Blake2b512, Digest};
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let mut h = Blake2b512::new();
            h.update(s.as_bytes());
            Ok(JpxValue::String(format!("{:x}", h.finalize())))
        }))),
        ("crc32", JpxValue::NativeFunction(Arc::new(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(JpxValue::Int(crc32fast::hash(s.as_bytes()) as i64))
        }))),
    ];

    let mut obj = HashMap::new();
    for (name, func) in fns {
        obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }
    interp.env.insert("hash".to_string(), JpxValue::Object(obj.clone()));
    interp.builtins.insert("hash".to_string(), JpxValue::Object(obj));
    Ok(())
}
