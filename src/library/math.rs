// math.rs - Math Library for JPX
// Rewritten from Python to Rust

use std::collections::HashMap;
use std::sync::Arc;
use crate::interpreter::interpreter::JpxInterpreter;
use crate::interpreter::exceptions::JpxError;
use crate::interpreter::value::JpxValue;

macro_rules! native_fn {
    ($name:expr, $body:expr) => {
        ($name, JpxValue::NativeFunction(Arc::new($body)))
    };
}

pub fn register(interp: &mut JpxInterpreter) -> Result<(), JpxError> {
    #[allow(unused_imports)]
    use std::sync::Arc;

    let fns: Vec<(&str, JpxValue)> = vec![
        // Constants
        ("pi", JpxValue::Float(std::f64::consts::PI)),
        ("e_const", JpxValue::Float(std::f64::consts::E)),
        ("tau", JpxValue::Float(std::f64::consts::TAU)),
        ("inf", JpxValue::Float(f64::INFINITY)),

        // Math functions
        native_fn!("abs_math", |args: Vec<JpxValue>| {
            let n = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            if n.fract() == 0.0 { Ok(JpxValue::Int(n.abs() as i64)) } else { Ok(JpxValue::Float(n.abs())) }
        }),
        native_fn!("ceil", |args: Vec<JpxValue>| {
            let n = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Int(n.ceil() as i64))
        }),
        native_fn!("floor", |args: Vec<JpxValue>| {
            let n = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Int(n.floor() as i64))
        }),
        native_fn!("round_math", |args: Vec<JpxValue>| {
            let n = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            let digits = args.get(1).and_then(|v| v.to_i64()).unwrap_or(0);
            let factor = 10f64.powi(digits as i32);
            Ok(JpxValue::Float((n * factor).round() / factor))
        }),
        native_fn!("sqrt", |args: Vec<JpxValue>| {
            let n = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(n.sqrt()))
        }),
        native_fn!("pow_math", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            let y = args.get(1).and_then(|v| v.to_f64()).unwrap_or(1.0);
            let r = x.powf(y);
            if r.fract() == 0.0 { Ok(JpxValue::Int(r as i64)) } else { Ok(JpxValue::Float(r)) }
        }),
        native_fn!("log", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(1.0);
            let base = args.get(1).and_then(|v| v.to_f64()).unwrap_or(std::f64::consts::E);
            Ok(JpxValue::Float(x.log(base)))
        }),
        native_fn!("log10", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(1.0);
            Ok(JpxValue::Float(x.log10()))
        }),
        native_fn!("log2", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(1.0);
            Ok(JpxValue::Float(x.log2()))
        }),
        native_fn!("exp", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(x.exp()))
        }),
        native_fn!("sin", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(x.sin()))
        }),
        native_fn!("cos", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(x.cos()))
        }),
        native_fn!("tan", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(x.tan()))
        }),
        native_fn!("asin", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(x.asin()))
        }),
        native_fn!("acos", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(x.acos()))
        }),
        native_fn!("atan", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(x.atan()))
        }),
        native_fn!("atan2", |args: Vec<JpxValue>| {
            let y = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            let x = args.get(1).and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(y.atan2(x)))
        }),
        native_fn!("degrees", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(x.to_degrees()))
        }),
        native_fn!("radians", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(x.to_radians()))
        }),
        native_fn!("hypot", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            let y = args.get(1).and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(x.hypot(y)))
        }),
        native_fn!("sinh", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(x.sinh()))
        }),
        native_fn!("cosh", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(x.cosh()))
        }),
        native_fn!("tanh", |args: Vec<JpxValue>| {
            let x = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Float(x.tanh()))
        }),
        native_fn!("factorial", |args: Vec<JpxValue>| {
            let n = args.first().and_then(|v| v.to_i64()).unwrap_or(0);
            if n < 0 { return Ok(JpxValue::Null); }
            let mut result: i64 = 1;
            for i in 2..=n { result *= i; }
            Ok(JpxValue::Int(result))
        }),
        native_fn!("gcd", |args: Vec<JpxValue>| {
            let a = args.first().and_then(|v| v.to_i64()).unwrap_or(0).abs();
            let b = args.get(1).and_then(|v| v.to_i64()).unwrap_or(0).abs();
            fn gcd(a: i64, b: i64) -> i64 { if b == 0 { a } else { gcd(b, a % b) } }
            Ok(JpxValue::Int(gcd(a, b)))
        }),
        native_fn!("min_math", |args: Vec<JpxValue>| {
            if args.is_empty() { return Ok(JpxValue::Null); }
            if args.len() == 1 {
                if let JpxValue::List(v) = &args[0] {
                    return Ok(v.iter().filter_map(|x| x.to_f64()).fold(f64::INFINITY, f64::min).into());
                }
            }
            let m = args.iter().filter_map(|v| v.to_f64()).fold(f64::INFINITY, f64::min);
            if m.fract() == 0.0 { Ok(JpxValue::Int(m as i64)) } else { Ok(JpxValue::Float(m)) }
        }),
        native_fn!("max_math", |args: Vec<JpxValue>| {
            if args.is_empty() { return Ok(JpxValue::Null); }
            if args.len() == 1 {
                if let JpxValue::List(v) = &args[0] {
                    return Ok(v.iter().filter_map(|x| x.to_f64()).fold(f64::NEG_INFINITY, f64::max).into());
                }
            }
            let m = args.iter().filter_map(|v| v.to_f64()).fold(f64::NEG_INFINITY, f64::max);
            if m.fract() == 0.0 { Ok(JpxValue::Int(m as i64)) } else { Ok(JpxValue::Float(m)) }
        }),
        native_fn!("sum_math", |args: Vec<JpxValue>| {
            let list = match args.first() {
                Some(JpxValue::List(v)) => v.clone(),
                _ => return Ok(JpxValue::Int(0)),
            };
            let total: f64 = list.iter().filter_map(|v| v.to_f64()).sum();
            if total.fract() == 0.0 { Ok(JpxValue::Int(total as i64)) } else { Ok(JpxValue::Float(total)) }
        }),
        native_fn!("avg", |args: Vec<JpxValue>| {
            let list = match args.first() {
                Some(JpxValue::List(v)) => v.clone(),
                _ => return Ok(JpxValue::Int(0)),
            };
            if list.is_empty() { return Ok(JpxValue::Int(0)); }
            let total: f64 = list.iter().filter_map(|v| v.to_f64()).sum();
            Ok(JpxValue::Float(total / list.len() as f64))
        }),
        native_fn!("isnan", |args: Vec<JpxValue>| {
            let n = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Bool(n.is_nan()))
        }),
        native_fn!("isinf", |args: Vec<JpxValue>| {
            let n = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Bool(n.is_infinite()))
        }),
        native_fn!("isfinite", |args: Vec<JpxValue>| {
            let n = args.first().and_then(|v| v.to_f64()).unwrap_or(0.0);
            Ok(JpxValue::Bool(n.is_finite()))
        }),
    ];

    // Register math object with all functions
    let mut math_obj = HashMap::new();
    math_obj.insert("pi".to_string(), JpxValue::Float(std::f64::consts::PI));
    math_obj.insert("e".to_string(), JpxValue::Float(std::f64::consts::E));
    math_obj.insert("tau".to_string(), JpxValue::Float(std::f64::consts::TAU));
    math_obj.insert("inf".to_string(), JpxValue::Float(f64::INFINITY));

    for (name, func) in &fns {
        math_obj.insert(name.to_string(), func.clone());
        interp.env.insert(name.to_string(), func.clone());
        interp.builtins.insert(name.to_string(), func.clone());
    }

    interp.env.insert("math".to_string(), JpxValue::Object(math_obj.clone()));
    interp.builtins.insert("math".to_string(), JpxValue::Object(math_obj));

    Ok(())
}

