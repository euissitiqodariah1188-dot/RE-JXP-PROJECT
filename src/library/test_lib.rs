// test_lib.rs - Test Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::collections::HashMap;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let mut obj = HashMap::new();
    obj.insert("hello".to_string(), JpxValue::String("Hello dari library test!".to_string()));
    obj.insert("get_message".to_string(), JpxValue::String("Ini pesan dari test".to_string()));
    interp.env.insert("test".to_string(), JpxValue::Object(obj.clone()));
    interp.builtins.insert("test".to_string(), JpxValue::Object(obj));
    Ok(())
}
