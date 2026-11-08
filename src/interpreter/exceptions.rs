// exceptions.rs - JPX Exception Types
// Rewritten from Python to Rust

use std::fmt;

#[derive(Debug, Clone)]
pub enum JpxError {
    Syntax(String),
    Name(String),
    Import(String),
    Type(String),
    Runtime(String),
}

impl fmt::Display for JpxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JpxError::Syntax(msg) => write!(f, "SyntaxError: {}", msg),
            JpxError::Name(msg) => write!(f, "NameError: {}", msg),
            JpxError::Import(msg) => write!(f, "ImportError: {}", msg),
            JpxError::Type(msg) => write!(f, "TypeError: {}", msg),
            JpxError::Runtime(msg) => write!(f, "RuntimeError: {}", msg),
        }
    }
}

impl std::error::Error for JpxError {}

/// Signal for break statement
#[derive(Debug, Clone)]
pub struct BreakSignal;

/// Signal for continue statement
#[derive(Debug, Clone)]
pub struct ContinueSignal;

/// Signal for return statement
#[derive(Debug, Clone)]
pub struct ReturnSignal {
    pub value: crate::interpreter::value::JpxValue,
}

/// Signal for throw/exception in try-catch
#[derive(Debug, Clone)]
pub struct ExceptionSignal {
    pub message: String,
    pub error_type: String,
}

impl ExceptionSignal {
    pub fn new(msg: impl Into<String>, kind: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            error_type: kind.into(),
        }
    }

    pub fn from_error(err: &JpxError) -> Self {
        let (kind, msg) = match err {
            JpxError::Syntax(m) => ("SyntaxError", m.clone()),
            JpxError::Name(m) => ("NameError", m.clone()),
            JpxError::Import(m) => ("ImportError", m.clone()),
            JpxError::Type(m) => ("TypeError", m.clone()),
            JpxError::Runtime(m) => ("RuntimeError", m.clone()),
        };
        Self {
            message: msg,
            error_type: kind.to_string(),
        }
    }
}
