// interpreter.rs - JPX Language Interpreter Core
// Rewritten from Python to Rust - v2.1.1 (bug-fixed)

use std::collections::HashMap;
use regex::Regex;

use crate::interpreter::value::{JpxValue, JpxFunction};
use crate::interpreter::exceptions::JpxError;
use crate::interpreter::importer::ImportHandler;

pub struct JpxInterpreter {
    pub env:          HashMap<String, JpxValue>,
    pub builtins:     HashMap<String, JpxValue>,
    pub functions:    HashMap<String, JpxFunction>,
    pub modules:      HashMap<String, HashMap<String, JpxValue>>,
    pub return_value: Option<JpxValue>,
    pub loop_depth:   usize,
}

impl JpxInterpreter {
    pub fn new() -> Self {
        let mut interp = Self {
            env:          HashMap::new(),
            builtins:     HashMap::new(),
            functions:    HashMap::new(),
            modules:      HashMap::new(),
            return_value: None,
            loop_depth:   0,
        };
        interp.builtins.insert("true".to_string(),  JpxValue::Bool(true));
        interp.builtins.insert("false".to_string(), JpxValue::Bool(false));
        interp.builtins.insert("null".to_string(),  JpxValue::Null);
        interp.env.insert("true".to_string(),  JpxValue::Bool(true));
        interp.env.insert("false".to_string(), JpxValue::Bool(false));
        interp.env.insert("null".to_string(),  JpxValue::Null);
        interp
    }

    // ─────────────────────────────────────────────────────────────
    // Public entry point
    // ─────────────────────────────────────────────────────────────
    pub fn run(&mut self, code: &str) -> Result<(), JpxError> {
        self.return_value = None;
        self.run_internal(code, false)
    }

    // ─────────────────────────────────────────────────────────────
    // Internal runner
    // ─────────────────────────────────────────────────────────────
    pub fn run_internal(&mut self, code: &str, _is_function: bool) -> Result<(), JpxError> {
        let code       = code.trim_start_matches('\u{FEFF}'); // strip BOM
        let clean      = self.strip_comments(code);
        let statements = self.split_statements(&clean);

        for stmt in statements {
            let stmt = stmt.trim().to_string();
            if stmt.is_empty() { continue; }

            match self.execute_statement(&stmt) {
                Ok(_) => {}
                Err(JpxError::Runtime(ref msg))
                    if msg == "__break__" || msg == "__continue__" || msg.starts_with("__return__") =>
                {
                    return Err(JpxError::Runtime(msg.clone()));
                }
                Err(JpxError::Runtime(ref msg)) if msg.starts_with("__exception__") => {
                    return Err(JpxError::Runtime(msg.clone()));
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────
    // Strip single-line comments (# and //)
    // ─────────────────────────────────────────────────────────────
    fn strip_comments(&self, code: &str) -> String {
        let mut lines = Vec::new();
        for line in code.lines() {
            let mut clean    = String::new();
            let mut in_str   = false;
            let mut qchar    = '"';
            let chars: Vec<char> = line.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                let ch = chars[i];
                if in_str {
                    clean.push(ch);
                    if ch == qchar && (i == 0 || chars[i - 1] != '\\') { in_str = false; }
                } else if (ch == '"' || ch == '\'') && (i == 0 || chars[i - 1] != '\\') {
                    in_str = true; qchar = ch; clean.push(ch);
                } else if ch == '#' { break; }
                else if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' { break; }
                else { clean.push(ch); }
                i += 1;
            }
            let t = clean.trim_end().to_string();
            if !t.is_empty() { lines.push(t); }
        }
        lines.join("\n")
    }

    // ─────────────────────────────────────────────────────────────
    // Split source into top-level statements
    // ─────────────────────────────────────────────────────────────
    pub fn split_statements(&self, code: &str) -> Vec<String> {
        let mut stmts    = Vec::new();
        let mut current  = String::new();
        let mut depth_br = 0i32; // [ ]
        let mut depth_bc = 0i32; // { }
        let mut in_str   = false;
        let mut qchar    = '"';
        let mut escape   = false;
        let chars: Vec<char> = code.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if in_str {
                current.push(ch);
                if ch == qchar && !escape { in_str = false; }
                escape = ch == '\\' && !escape;
                i += 1;
                continue;
            }
            escape = false;

            if ch == '"' || ch == '\'' { in_str = true; qchar = ch; current.push(ch); i += 1; continue; }
            if ch == '[' { depth_br += 1; current.push(ch); i += 1; continue; }
            if ch == ']' { depth_br -= 1; current.push(ch); i += 1; continue; }
            if ch == '{' { depth_bc += 1; current.push(ch); i += 1; continue; }
            if ch == '}' {
                depth_bc -= 1;
                current.push(ch);
                if depth_bc == 0 && depth_br == 0 {
                    // peek: if next non-whitespace is "catch", don't flush yet
                    let rest: String = chars[i + 1..].iter().collect();
                    if rest.trim_start().starts_with("catch") {
                        i += 1; continue;
                    }
                    let s = current.trim().to_string();
                    if !s.is_empty() { stmts.push(s); }
                    current = String::new();
                }
                i += 1;
                continue;
            }
            if ch == ';' && depth_br == 0 && depth_bc == 0 {
                let s = current.trim().to_string();
                if !s.is_empty() { stmts.push(s); }
                current = String::new();
                i += 1;
                continue;
            }

            current.push(ch);
            i += 1;
        }

        let s = current.trim().to_string();
        if !s.is_empty() { stmts.push(s); }
        stmts
    }

    // ─────────────────────────────────────────────────────────────
    // Execute a single statement
    // ─────────────────────────────────────────────────────────────
    fn execute_statement(&mut self, stmt: &str) -> Result<JpxValue, JpxError> {
        // try-catch
        if stmt.starts_with("try") && stmt.contains('{') {
            return self.handle_try_catch(stmt);
        }
        // throw / raise
        if stmt.starts_with("throw ") || stmt.starts_with("raise ") {
            let msg = stmt[6..].trim();
            let msg_val = self.eval_expression(msg)?;
            return Err(JpxError::Runtime(format!("__exception__: {}", msg_val)));
        }
        // function definition
        if stmt.starts_with("function ") || (stmt.contains('=') && stmt.contains("function(")) {
            return self.handle_function_def(stmt);
        }
        // return
        if stmt.starts_with("return ") || stmt == "return" {
            let val = self.handle_return(stmt)?;
            self.return_value = Some(val.clone());
            return Err(JpxError::Runtime("__return__".to_string()));
        }
        // import: [module] or [module.attr]
        if stmt.starts_with('[') && stmt.ends_with(']') && !stmt.contains('=') {
            self.handle_import(stmt)?;
            return Ok(JpxValue::Null);
        }
        // global [var = val]
        if stmt.starts_with("global [") {
            return self.handle_global(stmt);
        }
        // if
        if stmt.starts_with("if ") && stmt.contains('{') {
            return self.handle_if(stmt);
        }
        // while
        if stmt.starts_with("while ") && stmt.contains('{') {
            return self.handle_while(stmt);
        }
        // for
        if stmt.starts_with("for ") && stmt.contains('{') {
            return self.handle_for(stmt);
        }
        // break / continue
        if stmt == "break" {
            return if self.loop_depth > 0 {
                Err(JpxError::Runtime("__break__".to_string()))
            } else {
                Err(JpxError::Syntax("break outside loop".to_string()))
            };
        }
        if stmt == "continue" {
            return if self.loop_depth > 0 {
                Err(JpxError::Runtime("__continue__".to_string()))
            } else {
                Err(JpxError::Syntax("continue outside loop".to_string()))
            };
        }
        // end scanner (ignored)
        if Regex::new(r"^end\s+scanner\s*;?$").unwrap().is_match(stmt) {
            return Ok(JpxValue::Null);
        }

        self.eval_expression(stmt)
    }

    // ─────────────────────────────────────────────────────────────
    // try { ... } catch (e) { ... }
    // ─────────────────────────────────────────────────────────────
    fn handle_try_catch(&mut self, stmt: &str) -> Result<JpxValue, JpxError> {
        let try_brace = stmt.find('{')
            .ok_or_else(|| JpxError::Syntax("Missing '{' in try".to_string()))?;
        let (try_block, rest) = self.extract_brace_block(&stmt[try_brace..])?;
        let rest = rest.trim();

        if !rest.starts_with("catch") {
            return Err(JpxError::Syntax("Missing 'catch' after try block".to_string()));
        }
        let after_catch = rest["catch".len()..].trim();
        let p_start = after_catch.find('(')
            .ok_or_else(|| JpxError::Syntax("Missing '(' in catch".to_string()))?;
        let p_end   = after_catch.find(')')
            .ok_or_else(|| JpxError::Syntax("Missing ')' in catch".to_string()))?;
        let param   = after_catch[p_start + 1..p_end].trim().to_string();
        let param   = if param.is_empty() { "e".to_string() } else { param };

        let after_param  = after_catch[p_end + 1..].trim();
        let catch_brace  = after_param.find('{')
            .ok_or_else(|| JpxError::Syntax("Missing '{' in catch block".to_string()))?;
        let (catch_block, _) = self.extract_brace_block(&after_param[catch_brace..])?;

        match self.run_internal(&try_block, true) {
            Ok(_) => Ok(JpxValue::Null),
            // propagate control-flow signals unchanged
            Err(JpxError::Runtime(ref msg))
                if msg == "__break__" || msg == "__continue__" || msg.starts_with("__return__") =>
            {
                Err(JpxError::Runtime(msg.clone()))
            }
            Err(e) => {
                let (kind, msg) = match &e {
                    JpxError::Syntax(m)  => ("SyntaxError",  m.clone()),
                    JpxError::Name(m)    => ("NameError",    m.clone()),
                    JpxError::Import(m)  => ("ImportError",  m.clone()),
                    JpxError::Type(m)    => ("TypeError",    m.clone()),
                    JpxError::Runtime(m) => ("RuntimeError", m.clone()),
                };
                let mut exc = HashMap::new();
                exc.insert("message".to_string(), JpxValue::String(msg));
                exc.insert("type".to_string(),    JpxValue::String(kind.to_string()));

                let saved = self.env.clone();
                self.env.insert(param, JpxValue::Object(exc));
                let result = self.run_internal(&catch_block, true);
                self.env = saved;
                result.map(|_| JpxValue::Null)
            }
        }
    }

    // ─────────────────────────────────────────────────────────────
    // function name(params) { body }
    // ─────────────────────────────────────────────────────────────
    fn handle_function_def(&mut self, stmt: &str) -> Result<JpxValue, JpxError> {
        // Named function
        let re_named = Regex::new(r"^function\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(\s*(.*?)\s*\)\s*\{").unwrap();
        if let Some(caps) = re_named.captures(stmt) {
            let name    = caps[1].to_string();
            let params  = Self::parse_param_list(&caps[2]);
            let brace   = stmt.find('{').unwrap(); // safe: regex matched {
            let (body, _) = self.extract_brace_block(&stmt[brace..])?;
            self.register_function(Some(name), params, body);
            return Ok(JpxValue::Null);
        }

        // Anonymous: var = function(params) { body }
        if let Some(eq) = stmt.find('=') {
            let var_name = stmt[..eq].trim().to_string();
            let rest     = stmt[eq + 1..].trim();
            let re_anon  = Regex::new(r"^function\s*\(\s*(.*?)\s*\)\s*\{").unwrap();
            if let Some(caps) = re_anon.captures(rest) {
                let params = Self::parse_param_list(&caps[1]);
                let brace  = rest.find('{').unwrap(); // safe: regex matched {
                let (body, _) = self.extract_brace_block(&rest[brace..])?;
                self.register_function(Some(var_name), params, body);
                return Ok(JpxValue::Null);
            }
        }

        Err(JpxError::Syntax("Invalid function definition".to_string()))
    }

    fn parse_param_list(s: &str) -> Vec<String> {
        if s.trim().is_empty() { return vec![]; }
        s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
    }

    fn register_function(&mut self, name: Option<String>, params: Vec<String>, body: String) {
        let closure  = self.env.clone();
        let func     = JpxFunction::new(name.clone(), params, body, closure);
        let arc_func = std::sync::Arc::new(func.clone());
        let val      = JpxValue::Function(arc_func);
        if let Some(ref n) = name {
            self.env.insert(n.clone(), val.clone());
            self.functions.insert(n.clone(), func);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // return
    // ─────────────────────────────────────────────────────────────
    fn handle_return(&mut self, stmt: &str) -> Result<JpxValue, JpxError> {
        let expr = if stmt.len() > 7 { stmt[7..].trim() } else { "" };
        if expr.is_empty() { Ok(JpxValue::Null) }
        else               { self.eval_expression(expr) }
    }

    // ─────────────────────────────────────────────────────────────
    // global [var = value]
    // ─────────────────────────────────────────────────────────────
    fn handle_global(&mut self, stmt: &str) -> Result<JpxValue, JpxError> {
        let re   = Regex::new(r"^global\s*\[(.*)\]$").unwrap();
        let caps = re.captures(stmt)
            .ok_or_else(|| JpxError::Syntax("Invalid global statement".to_string()))?;
        let inner  = caps[1].trim().trim_end_matches(';').trim();
        let eq_pos = inner.find('=')
            .ok_or_else(|| JpxError::Syntax("Missing '=' in global".to_string()))?;
        let var_name = inner[..eq_pos].trim().to_string();
        let val_expr = inner[eq_pos + 1..].trim();
        let value    = self.eval_expression(val_expr)?;
        // Auto-coerce string → number
        let value = match &value {
            JpxValue::String(s) => {
                if let Ok(n) = s.parse::<i64>()  { JpxValue::Int(n)   }
                else if let Ok(f) = s.parse::<f64>() { JpxValue::Float(f) }
                else { value }
            }
            _ => value,
        };
        self.env.insert(var_name, value.clone());
        Ok(value)
    }

    // ─────────────────────────────────────────────────────────────
    // Variable lookup (dot notation, array access)
    // &mut self because it may call eval_expression for arr[i]
    // ─────────────────────────────────────────────────────────────
    pub fn get_value(&mut self, ident: &str) -> Result<JpxValue, JpxError> {
        // Array/computed access
        if ident.contains('[') {
            return self.eval_expression(ident);
        }

        let parts: Vec<String> = ident.splitn(2, '.').map(|s| s.to_string()).collect();
        let base  = &parts[0];

        // Lookup in env → builtins → functions
        let base_val = if let Some(v) = self.env.get(base).cloned() {
            v
        } else if let Some(v) = self.builtins.get(base).cloned() {
            v
        } else if let Some(f) = self.functions.get(base).cloned() {
            JpxValue::Function(std::sync::Arc::new(f))
        } else {
            return Err(JpxError::Name(format!("Undefined variable: '{}'", base)));
        };

        if parts.len() == 1 { return Ok(base_val); }

        // Navigate remaining dot segments
        let mut current = base_val;
        for part in &parts[1..] {
            current = match current {
                JpxValue::Object(ref m) => m.get(part).cloned().unwrap_or(JpxValue::Null),
                _                       => JpxValue::Null,
            };
        }
        Ok(current)
    }

    // ─────────────────────────────────────────────────────────────
    // if / else
    // ─────────────────────────────────────────────────────────────
    fn handle_if(&mut self, stmt: &str) -> Result<JpxValue, JpxError> {
        let brace  = stmt.find('{')
            .ok_or_else(|| JpxError::Syntax("Missing '{' in if".to_string()))?;
        let cond   = stmt[3..brace].trim().to_string();
        let (block_if, rest) = self.extract_brace_block(&stmt[brace..])?;
        let rest   = rest.trim();

        let block_else: Option<String> = if rest.starts_with("else") {
            let after = rest["else".len()..].trim();
            if after.starts_with("if ") {
                // else-if chain
                Some(after.to_string())
            } else if after.starts_with('{') {
                let (b, _) = self.extract_brace_block(after)?;
                Some(b)
            } else {
                Some(after.to_string())
            }
        } else {
            None
        };

        let cond_val = self.eval_expression(&cond)?;
        if cond_val.is_truthy() {
            self.run_internal(&block_if, true)?;
        } else if let Some(else_block) = block_else {
            self.run_internal(&else_block, true)?;
        }
        Ok(JpxValue::Null)
    }

    // ─────────────────────────────────────────────────────────────
    // while loop
    // BUG FIX: loop_depth is always decremented even if eval fails
    // ─────────────────────────────────────────────────────────────
    fn handle_while(&mut self, stmt: &str) -> Result<JpxValue, JpxError> {
        let brace  = stmt.find('{')
            .ok_or_else(|| JpxError::Syntax("Missing '{' in while".to_string()))?;
        let cond   = stmt[6..brace].trim().to_string();
        let (block, _) = self.extract_brace_block(&stmt[brace..])?;

        self.loop_depth += 1;
        let result = self.while_loop_body(&cond, &block);
        self.loop_depth -= 1; // ALWAYS decrement, even on error
        result
    }

    fn while_loop_body(&mut self, cond: &str, block: &str) -> Result<JpxValue, JpxError> {
        loop {
            let cond_val = self.eval_expression(cond)?;
            if !cond_val.is_truthy() { break; }
            match self.run_internal(block, true) {
                Ok(_)                                                              => {}
                Err(JpxError::Runtime(ref msg)) if msg == "__break__"             => break,
                Err(JpxError::Runtime(ref msg)) if msg == "__continue__"          => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(JpxValue::Null)
    }

    // ─────────────────────────────────────────────────────────────
    // for loop — range: for x = a to b  |  iter: for x in list
    // BUG FIX: loop_depth always decremented via helper
    // ─────────────────────────────────────────────────────────────
    fn handle_for(&mut self, stmt: &str) -> Result<JpxValue, JpxError> {
        // Range: for var = start to end { ... }
        let re_range = Regex::new(
            r"^for\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+?)\s+to\s+(.+?)\s*\{"
        ).unwrap();

        if let Some(caps) = re_range.captures(stmt) {
            let var_name   = caps[1].to_string();
            let start_expr = caps[2].trim().to_string();
            let end_expr   = caps[3].trim().to_string();
            let brace      = stmt.find('{').unwrap();
            let (block, _) = self.extract_brace_block(&stmt[brace..])?;

            let start = self.eval_expression(&start_expr)?.to_i64()
                .ok_or_else(|| JpxError::Type("for range start must be a number".to_string()))?;
            let end   = self.eval_expression(&end_expr)?.to_i64()
                .ok_or_else(|| JpxError::Type("for range end must be a number".to_string()))?;

            self.loop_depth += 1;
            let result = self.for_range_body(&var_name, start, end, &block);
            self.loop_depth -= 1;
            return result;
        }

        // Iterator: for var in list { ... }
        let re_iter = Regex::new(r"^for\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+in\s+(.+?)\s*\{").unwrap();
        if let Some(caps) = re_iter.captures(stmt) {
            let var_name   = caps[1].to_string();
            let list_expr  = caps[2].trim().to_string();
            let brace      = stmt.find('{').unwrap();
            let (block, _) = self.extract_brace_block(&stmt[brace..])?;

            let list_val = self.eval_expression(&list_expr)?;
            let items    = match list_val {
                JpxValue::List(v) => v,
                other             => return Err(JpxError::Type(format!(
                    "'for in' requires a list, got {}", other.type_name()
                ))),
            };

            self.loop_depth += 1;
            let result = self.for_iter_body(&var_name, items, &block);
            self.loop_depth -= 1;
            return result;
        }

        Err(JpxError::Syntax("Invalid for loop syntax".to_string()))
    }

    fn for_range_body(&mut self, var: &str, start: i64, end: i64, block: &str)
        -> Result<JpxValue, JpxError>
    {
        let mut i = start;
        loop {
            if i > end { break; }
            self.env.insert(var.to_string(), JpxValue::Int(i));
            match self.run_internal(block, true) {
                Ok(_)                                                     => {}
                Err(JpxError::Runtime(ref msg)) if msg == "__break__"    => break,
                Err(JpxError::Runtime(ref msg)) if msg == "__continue__" => { i += 1; continue; }
                Err(e) => return Err(e),
            }
            i += 1;
        }
        Ok(JpxValue::Null)
    }

    fn for_iter_body(&mut self, var: &str, items: Vec<JpxValue>, block: &str)
        -> Result<JpxValue, JpxError>
    {
        for item in items {
            self.env.insert(var.to_string(), item);
            match self.run_internal(block, true) {
                Ok(_)                                                     => {}
                Err(JpxError::Runtime(ref msg)) if msg == "__break__"    => break,
                Err(JpxError::Runtime(ref msg)) if msg == "__continue__" => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(JpxValue::Null)
    }

    // ─────────────────────────────────────────────────────────────
    // Extract { ... } block from start of string
    // Returns (content_inside, rest_after_closing_brace)
    // ─────────────────────────────────────────────────────────────
    pub fn extract_brace_block(&self, s: &str) -> Result<(String, String), JpxError> {
        if !s.starts_with('{') {
            return Err(JpxError::Syntax("Expected '{'".to_string()));
        }
        let mut depth:   i32  = 0;
        let mut in_str:  bool = false;
        let mut qchar:   char = '"';

        for (byte_pos, ch) in s.char_indices() {
            if in_str {
                if ch == qchar { in_str = false; }
                continue;
            }
            if ch == '"' || ch == '\'' { in_str = true; qchar = ch; continue; }
            if ch == '{' { depth += 1; }
            else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    // byte_pos is the index of the closing '}'
                    let block = s[1..byte_pos].to_string();
                    let rest  = s[byte_pos + 1..].trim_start().to_string();
                    return Ok((block, rest));
                }
            }
        }
        Err(JpxError::Syntax("Unclosed '{' block".to_string()))
    }

    // ─────────────────────────────────────────────────────────────
    // Import handler
    // ─────────────────────────────────────────────────────────────
    fn handle_import(&mut self, stmt: &str) -> Result<(), JpxError> {
        let name = stmt[1..stmt.len() - 1].trim().to_string();
        ImportHandler::new().handle(self, &name)
    }
}
