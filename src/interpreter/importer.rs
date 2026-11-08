// importer.rs - JPX Module Import Handler
// Rewritten from Python to Rust
// Loads built-in library modules into the interpreter environment

use std::path::PathBuf;
use crate::interpreter::interpreter::JpxInterpreter;
use crate::interpreter::exceptions::JpxError;
use crate::library;

pub struct ImportHandler {
    pub builtin_path: PathBuf,
    pub user_path: PathBuf,
}

impl ImportHandler {
    pub fn new() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        Self {
            builtin_path: exe_dir.join("library"),
            user_path: cwd.join("lib"),
        }
    }

    /// Handle import statement: [module] or [module.attribute]
    pub fn handle(&self, interp: &mut JpxInterpreter, name: &str) -> Result<(), JpxError> {
        if name.contains('.') {
            let parts: Vec<&str> = name.splitn(2, '.').collect();
            let module_name = parts[0];
            let attr_name = parts[1];
            self.load_module_attr(interp, module_name, attr_name)
        } else {
            self.load_module(interp, name)
        }
    }

    /// Load a full module by name and inject its exports into the interpreter
    fn load_module(&self, interp: &mut JpxInterpreter, name: &str) -> Result<(), JpxError> {
        match name {
            "math" => library::math::register(interp),
            "random" => library::random::register(interp),
            "hash" => library::hash::register(interp),
            "fs" => library::fs::register(interp),
            "json" => library::json_lib::register(interp),
            "string" => library::string::register(interp),
            "time" => library::time::register(interp),
            "requests" => library::requests::register(interp),
            "speedreq" => library::speedreq::register(interp),
            "scanner" => library::scanner::register(interp),
            "env" => library::env::register(interp),
            "system" => library::system::register(interp),
            "color" => library::color::register(interp),
            "JpXhash" => library::jpxhash::register(interp),
            "ReGx" => library::regex_lib::register(interp),
            "beuty" | "beauty" => library::beuty::register(interp),
            "notification" | "subnotif" => library::notification::register(interp),
            "print" => library::print_lib::register(interp),
            "test" => library::test_lib::register(interp),
            _ => Err(JpxError::Import(format!(
                "Module '{}' not found.\n  Looked in: {:?}\n  User path: {:?}",
                name, self.builtin_path, self.user_path
            ))),
        }
    }

    /// Load a specific attribute from a module
    fn load_module_attr(&self, interp: &mut JpxInterpreter, module: &str, attr: &str) -> Result<(), JpxError> {
        // Load the full module first (into a temp scope)
        let mut temp_interp = JpxInterpreter::new();
        self.load_module(&mut temp_interp, module)?;

        // Extract the requested attribute
        if let Some(val) = temp_interp.env.get(attr).cloned().or_else(|| temp_interp.builtins.get(attr).cloned()) {
            interp.env.insert(attr.to_string(), val.clone());
            interp.builtins.insert(attr.to_string(), val);
            Ok(())
        } else {
            Err(JpxError::Import(format!("Attribute '{}' not found in module '{}'", attr, module)))
        }
    }
}
