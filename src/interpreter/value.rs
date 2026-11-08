// value.rs - JPX Value Type System
// Rewritten from Python to Rust - v2.1.1 (bug-fixed)

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// Central value type for all JPX runtime values
#[derive(Clone)]
pub enum JpxValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<JpxValue>),
    Object(HashMap<String, JpxValue>),
    Function(Arc<JpxFunction>),
    NativeFunction(Arc<dyn Fn(Vec<JpxValue>) -> Result<JpxValue, crate::interpreter::exceptions::JpxError> + Send + Sync>),
    NativeObject(Arc<Mutex<Box<dyn NativeObject + Send>>>),
}

// Manual Debug: dyn Fn doesn't implement Debug
impl fmt::Debug for JpxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JpxValue::Null              => write!(f, "Null"),
            JpxValue::Bool(b)           => write!(f, "Bool({:?})", b),
            JpxValue::Int(n)            => write!(f, "Int({})", n),
            JpxValue::Float(n)          => write!(f, "Float({})", n),
            JpxValue::String(s)         => write!(f, "String({:?})", s),
            JpxValue::List(v)           => write!(f, "List({:?})", v),
            JpxValue::Object(m)         => write!(f, "Object({:?})", m),
            JpxValue::Function(func)    => write!(f, "Function({:?})", func.name),
            JpxValue::NativeFunction(_) => write!(f, "NativeFunction(<fn>)"),
            JpxValue::NativeObject(_)   => write!(f, "NativeObject(<obj>)"),
        }
    }
}

impl PartialEq for JpxValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JpxValue::Null,          JpxValue::Null)          => true,
            (JpxValue::Bool(a),       JpxValue::Bool(b))       => a == b,
            (JpxValue::Int(a),        JpxValue::Int(b))        => a == b,
            (JpxValue::Float(a),      JpxValue::Float(b))      => a == b,
            (JpxValue::Int(a),        JpxValue::Float(b))      => (*a as f64) == *b,
            (JpxValue::Float(a),      JpxValue::Int(b))        => *a == (*b as f64),
            (JpxValue::String(a),     JpxValue::String(b))     => a == b,
            (JpxValue::List(a),       JpxValue::List(b))       => a == b,
            // Bool vs Int coercion (matches Python behaviour)
            (JpxValue::Bool(a),       JpxValue::Int(b))        => (*a as i64) == *b,
            (JpxValue::Int(a),        JpxValue::Bool(b))       => *a == (*b as i64),
            _                                                    => false,
        }
    }
}

impl PartialOrd for JpxValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (JpxValue::Int(a),    JpxValue::Int(b))    => a.partial_cmp(b),
            (JpxValue::Float(a),  JpxValue::Float(b))  => a.partial_cmp(b),
            (JpxValue::Int(a),    JpxValue::Float(b))  => (*a as f64).partial_cmp(b),
            (JpxValue::Float(a),  JpxValue::Int(b))    => a.partial_cmp(&(*b as f64)),
            (JpxValue::String(a), JpxValue::String(b)) => a.partial_cmp(b),
            _                                            => None,
        }
    }
}

impl fmt::Display for JpxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JpxValue::Null        => write!(f, "null"),
            JpxValue::Bool(b)     => write!(f, "{}", if *b { "true" } else { "false" }),
            JpxValue::Int(n)      => write!(f, "{}", n),
            JpxValue::Float(n)    => {
                // Avoid printing "1.0" as "1" — keep decimal point
                if n.is_finite() && n.fract() == 0.0 && n.abs() < 1e15 {
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            JpxValue::String(s)   => write!(f, "{}", s),
            JpxValue::List(v)     => {
                let parts: Vec<std::string::String> = v.iter().map(|x| format!("{}", x)).collect();
                write!(f, "[{}]", parts.join(", "))
            }
            JpxValue::Object(m)   => {
                let parts: Vec<std::string::String> = m.iter()
                    .map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", parts.join(", "))
            }
            JpxValue::Function(func) => {
                if let Some(name) = &func.name { write!(f, "<function {}>", name) }
                else                            { write!(f, "<anonymous function>") }
            }
            JpxValue::NativeFunction(_) => write!(f, "<native function>"),
            JpxValue::NativeObject(_)   => write!(f, "<native object>"),
        }
    }
}

impl JpxValue {
    /// JPX truthiness: null/false/0/"" are falsy
    pub fn is_truthy(&self) -> bool {
        match self {
            JpxValue::Null        => false,
            JpxValue::Bool(b)     => *b,
            JpxValue::Int(n)      => *n != 0,
            JpxValue::Float(n)    => *n != 0.0,
            JpxValue::String(s)   => {
                match s.to_lowercase().as_str() {
                    "true"  => return true,
                    "false" => return false,
                    _       => {}
                }
                !s.is_empty()
            }
            JpxValue::List(v)     => !v.is_empty(),
            JpxValue::Object(m)   => !m.is_empty(),
            _                     => true,
        }
    }

    pub fn to_f64(&self) -> Option<f64> {
        match self {
            JpxValue::Int(n)    => Some(*n as f64),
            JpxValue::Float(n)  => Some(*n),
            JpxValue::String(s) => s.parse::<f64>().ok(),
            JpxValue::Bool(b)   => Some(if *b { 1.0 } else { 0.0 }),
            _                   => None,
        }
    }

    pub fn to_i64(&self) -> Option<i64> {
        match self {
            JpxValue::Int(n)    => Some(*n),
            // BUG FIX: cast via i64 to avoid UB on very large floats
            JpxValue::Float(n)  => {
                if n.is_finite() { Some(*n as i64) } else { None }
            }
            JpxValue::String(s) => s.trim().parse::<i64>().ok(),
            JpxValue::Bool(b)   => Some(if *b { 1 } else { 0 }),
            _                   => None,
        }
    }

    pub fn len(&self) -> Option<usize> {
        match self {
            JpxValue::String(s) => Some(s.chars().count()), // char count, not bytes
            JpxValue::List(v)   => Some(v.len()),
            JpxValue::Object(m) => Some(m.len()),
            _                   => None,
        }
    }

    /// Index into a list (supports negative indices) or object
    pub fn index(&self, key: &JpxValue) -> Option<JpxValue> {
        match (self, key) {
            (JpxValue::List(v), JpxValue::Int(i)) => {
                let len = v.len() as i64;
                // BUG FIX: guard against out-of-bounds
                let idx = if *i < 0 { len + i } else { *i };
                if idx >= 0 && idx < len { v.get(idx as usize).cloned() }
                else { None }
            }
            (JpxValue::List(v), JpxValue::Float(f)) => {
                let idx = *f as usize;
                v.get(idx).cloned()
            }
            (JpxValue::Object(m), JpxValue::String(k)) => m.get(k).cloned(),
            (JpxValue::Object(m), JpxValue::Int(i))    => m.get(&i.to_string()).cloned(),
            (JpxValue::String(s), JpxValue::Int(i))    => {
                // string character indexing
                let len = s.chars().count() as i64;
                let idx = if *i < 0 { len + i } else { *i };
                if idx >= 0 && idx < len {
                    s.chars().nth(idx as usize).map(|c| JpxValue::String(c.to_string()))
                } else { None }
            }
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            JpxValue::Null             => "null",
            JpxValue::Bool(_)          => "bool",
            JpxValue::Int(_)           => "int",
            JpxValue::Float(_)         => "float",
            JpxValue::String(_)        => "string",
            JpxValue::List(_)          => "list",
            JpxValue::Object(_)        => "object",
            JpxValue::Function(_)      => "function",
            JpxValue::NativeFunction(_)=> "function",
            JpxValue::NativeObject(_)  => "object",
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// JpxFunction
// ─────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct JpxFunction {
    pub name:        Option<String>,
    pub params:      Vec<String>,
    pub body:        String,
    pub closure_env: HashMap<String, JpxValue>,
}

impl JpxFunction {
    pub fn new(name: Option<String>, params: Vec<String>, body: String,
               closure_env: HashMap<String, JpxValue>) -> Self
    {
        Self { name, params, body, closure_env }
    }
}

// ─────────────────────────────────────────────────────────────────
// NativeObject trait
// ─────────────────────────────────────────────────────────────────
pub trait NativeObject: fmt::Debug {
    fn get_attr(&self, name: &str) -> Option<JpxValue>;
    fn set_attr(&mut self, name: &str, value: JpxValue);
    fn call_method(&mut self, name: &str, args: Vec<JpxValue>)
        -> Result<JpxValue, crate::interpreter::exceptions::JpxError>;
    fn type_name(&self) -> &str;
}

// ─────────────────────────────────────────────────────────────────
// Convenience conversions
// ─────────────────────────────────────────────────────────────────
impl From<f64>   for JpxValue { fn from(f: f64)   -> Self { JpxValue::Float(f)  } }
impl From<i64>   for JpxValue { fn from(n: i64)   -> Self { JpxValue::Int(n)    } }
impl From<bool>  for JpxValue { fn from(b: bool)  -> Self { JpxValue::Bool(b)   } }
impl From<String> for JpxValue { fn from(s: String) -> Self { JpxValue::String(s) } }
impl From<&str>  for JpxValue { fn from(s: &str)  -> Self { JpxValue::String(s.to_string()) } }
