// print_lib.rs - Print Library for JPX (rewritten from Python)
use crate::interpreter::{interpreter::JpxInterpreter, exceptions::JpxError, value::JpxValue};
use std::sync::Arc;

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    let print_fn = JpxValue::NativeFunction(Arc::new(|args| {
        let s = args.first().map(|v| v.to_string()).unwrap_or_default();
        println!("{}", s);
        Ok(JpxValue::Null)
    }));
    interp.env.insert("print".to_string(), print_fn.clone());
    interp.builtins.insert("print".to_string(), print_fn);
    Ok(())
}
