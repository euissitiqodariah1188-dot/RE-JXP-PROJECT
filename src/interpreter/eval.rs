// eval.rs - JPX Expression Evaluator
// Rewritten from Python to Rust - v2.1.1 (bug-fixed)

use std::collections::HashMap;
use regex::Regex;

use crate::interpreter::value::JpxValue;
use crate::interpreter::exceptions::JpxError;
use crate::interpreter::interpreter::JpxInterpreter;

impl JpxInterpreter {
    // ─────────────────────────────────────────────────────────────
    // String interpolation  ($var inside "...")
    // ─────────────────────────────────────────────────────────────
    /// Process string interpolation: $variable inside strings.
    /// Two-pass approach avoids needing &mut self inside the closure.
    pub fn process_string(&mut self, s: &str) -> String {
        let re = Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_.]*)").unwrap();

        // Pass 1: collect every var name referenced
        let var_names: Vec<String> = re
            .captures_iter(s)
            .map(|caps| caps[1].to_string())
            .collect();

        // Pass 2: resolve each var (needs &mut self)
        let mut resolved: HashMap<String, String> = HashMap::new();
        for name in &var_names {
            let val_str = match self.get_value(name) {
                Ok(v)  => v.to_string(),
                Err(_) => format!("<undefined:{}>", name),
            };
            resolved.insert(name.clone(), val_str);
        }

        // Pass 3: substitute — only captures resolved map, no &mut self
        re.replace_all(s, |caps: &regex::Captures| {
            resolved
                .get(&caps[1])
                .cloned()
                .unwrap_or_else(|| format!("<undefined:{}>", &caps[1]))
        })
        .into_owned()
    }

    // ─────────────────────────────────────────────────────────────
    // Main expression evaluator
    // ─────────────────────────────────────────────────────────────
    pub fn eval_expression(&mut self, expr: &str) -> Result<JpxValue, JpxError> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Ok(JpxValue::Null);
        }

        // ── Double-quoted string ──────────────────────────────────
        if expr.starts_with('"') && expr.ends_with('"') && expr.len() >= 2 {
            // Safe: '"' is always 1 byte in UTF-8
            let inner = &expr[1..expr.len() - 1];
            return Ok(JpxValue::String(self.process_string(inner)));
        }

        // ── Triple-quoted string ──────────────────────────────────
        if expr.starts_with("\"\"\"") && expr.ends_with("\"\"\"") && expr.len() >= 6 {
            let inner = &expr[3..expr.len() - 3];
            return Ok(JpxValue::String(self.process_string(inner)));
        }

        // ── Single-quoted string ──────────────────────────────────
        if expr.starts_with('\'') && expr.ends_with('\'') && expr.len() >= 2 {
            let inner = &expr[1..expr.len() - 1];
            return Ok(JpxValue::String(inner.to_string()));
        }

        // ── Ternary operator ──────────────────────────────────────
        if expr.contains('?') && expr.contains(':') {
            if let Some(result) = self.try_eval_ternary(expr)? {
                return Ok(result);
            }
        }

        // ── Control-flow keywords as expressions ──────────────────
        if expr == "break" || expr == "broke" {
            return Err(JpxError::Runtime("__break__".to_string()));
        }
        if expr == "continue" {
            return Err(JpxError::Runtime("__continue__".to_string()));
        }

        // ── print statement ───────────────────────────────────────
        if expr.starts_with("print ") {
            let content = expr[6..].trim();
            let val = self.eval_print_content(content)?;
            println!("{}", val);
            return Ok(JpxValue::Null);
        }

        // ── Parenthesised expression ──────────────────────────────
        if expr.starts_with('(') && expr.ends_with(')') && self.matching_paren(expr) {
            return self.eval_expression(&expr[1..expr.len() - 1]);
        }

        // ── Object literal ────────────────────────────────────────
        if expr.starts_with('{') && expr.ends_with('}') {
            return self.parse_object(expr);
        }

        // ── List literal ──────────────────────────────────────────
        if expr.starts_with('[') && expr.ends_with(']') {
            return self.parse_list(expr);
        }

        // ── Function call ─────────────────────────────────────────
        if let Some(result) = self.try_eval_func_call(expr)? {
            return Ok(result);
        }

        // ── Assignment ────────────────────────────────────────────
        if let Some(result) = self.try_eval_assignment(expr)? {
            return Ok(result);
        }

        // ── Nested index: arr[0][1] ───────────────────────────────
        let re_nested = Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_.]*)(\[(\d+)\])(\[(\d+)\])$").unwrap();
        if let Some(caps) = re_nested.captures(expr) {
            let name   = &caps[1];
            let idx1: usize = caps[3].parse().unwrap_or(0);
            let idx2: usize = caps[5].parse().unwrap_or(0);
            let obj = self.get_value(name)?;
            if let JpxValue::List(v) = obj {
                if let Some(JpxValue::List(inner)) = v.get(idx1) {
                    return Ok(inner.get(idx2).cloned().unwrap_or(JpxValue::Null));
                }
            }
            return Ok(JpxValue::Null);
        }

        // ── Index access: name[key] ───────────────────────────────
        let re_index = Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_.]*)(\[(.+?)\])$").unwrap();
        if let Some(caps) = re_index.captures(expr) {
            let obj_name = caps[1].to_string();
            let key_expr = caps[3].trim().to_string();
            let obj = self.get_value(&obj_name)?;
            let key = self.eval_expression(&key_expr)?;
            return match obj.index(&key) {
                Some(v) => Ok(v),
                None    => Ok(JpxValue::Null), // graceful: return null instead of panic
            };
        }

        // ── Length operator: "var length" ─────────────────────────
        if expr.ends_with(" length") {
            let var_name = &expr[..expr.len() - 7];
            let val = self.eval_expression(var_name)?;
            return match val.len() {
                Some(n) => Ok(JpxValue::Int(n as i64)),
                None    => Err(JpxError::Type(format!("Cannot get length of {}", val.type_name()))),
            };
        }

        // ── Logical operators ─────────────────────────────────────
        if self.contains_op_outside_strings(expr, " and ") {
            let parts = self.split_on_op(expr, " and ");
            let left  = self.eval_expression(parts[0].trim())?;
            if !left.is_truthy() { return Ok(JpxValue::Bool(false)); }
            let right = self.eval_expression(parts[1].trim())?;
            return Ok(JpxValue::Bool(right.is_truthy()));
        }
        if self.contains_op_outside_strings(expr, " or ") {
            let parts = self.split_on_op(expr, " or ");
            let left  = self.eval_expression(parts[0].trim())?;
            if left.is_truthy() { return Ok(JpxValue::Bool(true)); }
            let right = self.eval_expression(parts[1].trim())?;
            return Ok(JpxValue::Bool(right.is_truthy()));
        }
        if expr.starts_with("not ") {
            let val = self.eval_expression(&expr[4..])?;
            return Ok(JpxValue::Bool(!val.is_truthy()));
        }

        // ── Comparison operators ──────────────────────────────────
        for op in &["<=", ">=", "==", "!=", "<", ">"] {
            if let Some(result) = self.try_eval_comparison(expr, op)? {
                return Ok(result);
            }
        }

        // ── Arithmetic ────────────────────────────────────────────
        if let Some(r) = self.try_eval_addition(expr)?    { return Ok(r); }
        if let Some(r) = self.try_eval_subtraction(expr)? { return Ok(r); }
        if let Some(r) = self.try_eval_mul_div(expr)?     { return Ok(r); }
        if let Some(r) = self.try_eval_modulo(expr)?      { return Ok(r); }

        // ── Unary minus ───────────────────────────────────────────
        if expr.starts_with('-') {
            let val = self.eval_expression(&expr[1..])?;
            return match val.to_f64() {
                Some(f) => if f.fract() == 0.0 { Ok(JpxValue::Int(-(f as i64))) }
                           else { Ok(JpxValue::Float(-f)) },
                None    => Err(JpxError::Type("Cannot negate non-number".to_string())),
            };
        }

        // ── Literals ──────────────────────────────────────────────
        if let Ok(n) = expr.parse::<i64>()  { return Ok(JpxValue::Int(n));   }
        if let Ok(f) = expr.parse::<f64>()  { return Ok(JpxValue::Float(f)); }
        if expr == "true"  { return Ok(JpxValue::Bool(true));  }
        if expr == "false" { return Ok(JpxValue::Bool(false)); }
        if expr == "null"  { return Ok(JpxValue::Null);        }

        // ── Dot-notation property access ──────────────────────────
        if expr.contains('.') && !expr.contains('(') {
            if let Ok(val) = self.get_value(expr) { return Ok(val); }
        }

        // ── Variable lookup ───────────────────────────────────────
        let ident_re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_.]*$").unwrap();
        if ident_re.is_match(expr) {
            return match self.get_value(expr) {
                Ok(v) => Ok(v),
                Err(JpxError::Name(msg)) => Err(JpxError::Name(
                    format!("{} — Use 'global [{} = value]' to declare it.", msg, expr)
                )),
                Err(e) => Err(e),
            };
        }

        Err(JpxError::Syntax(format!("Invalid expression: '{}'", expr)))
    }

    // ─────────────────────────────────────────────────────────────
    // Helper: split on the first occurrence of op outside strings
    // ─────────────────────────────────────────────────────────────
    fn split_on_op<'a>(&self, expr: &'a str, op: &str) -> Vec<&'a str> {
        if let Some(pos) = self.find_op_pos_outside_strings(expr, op) {
            vec![&expr[..pos], &expr[pos + op.len()..]]
        } else {
            vec![expr]
        }
    }

    fn find_op_pos_outside_strings(&self, expr: &str, op: &str) -> Option<usize> {
        let op_bytes = op.as_bytes();
        let bytes = expr.as_bytes();
        let mut in_string = false;
        let mut quote_char = b'"';
        let mut depth: i32 = 0;
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if in_string {
                if b == quote_char && (i == 0 || bytes[i - 1] != b'\\') { in_string = false; }
            } else if b == b'"' || b == b'\'' {
                in_string = true; quote_char = b;
            } else if b == b'(' || b == b'[' || b == b'{' { depth += 1; }
            else if b == b')' || b == b']' || b == b'}' { depth -= 1; }
            else if depth == 0 && i + op_bytes.len() <= bytes.len()
                && &bytes[i..i + op_bytes.len()] == op_bytes {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    /// Check if op appears outside strings/parens
    fn contains_op_outside_strings(&self, expr: &str, op: &str) -> bool {
        self.find_op_pos_outside_strings(expr, op).is_some()
    }

    // ─────────────────────────────────────────────────────────────
    // Ternary: cond ? true_val : false_val
    // ─────────────────────────────────────────────────────────────
    fn try_eval_ternary(&mut self, expr: &str) -> Result<Option<JpxValue>, JpxError> {
        let q_pos = match self.find_op_pos_outside_strings(expr, "?") {
            Some(p) => p,
            None    => return Ok(None),
        };
        let condition = &expr[..q_pos];
        let rest      = expr[q_pos + 1..].trim();
        let c_pos     = match self.find_op_pos_outside_strings(rest, ":") {
            Some(p) => p,
            None    => return Err(JpxError::Syntax("Ternary missing ':'".to_string())),
        };
        let true_val  = rest[..c_pos].trim();
        let false_val = rest[c_pos + 1..].trim();
        let cond      = self.eval_expression(condition.trim())?;
        if cond.is_truthy() { Ok(Some(self.eval_expression(true_val)?)) }
        else                 { Ok(Some(self.eval_expression(false_val)?)) }
    }

    // ─────────────────────────────────────────────────────────────
    // Function call: name(args) or obj.method(args)
    // ─────────────────────────────────────────────────────────────
    fn try_eval_func_call(&mut self, expr: &str) -> Result<Option<JpxValue>, JpxError> {
        let re = Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_.]*)(\(.*\))$").unwrap();
        if !re.is_match(expr) { return Ok(None); }

        let paren_start = match expr.find('(') {
            Some(p) => p,
            None    => return Ok(None),
        };
        let func_path = &expr[..paren_start];
        // Safety: paren_start is a '(' which is 1 byte; expr ends with ')' which is 1 byte
        let args_str  = &expr[paren_start + 1..expr.len() - 1];
        let args      = self.parse_args(args_str)?;

        // Dotted method call: obj.method(args)
        if func_path.contains('.') {
            let dot_pos    = func_path.rfind('.').unwrap();
            let obj_name   = &func_path[..dot_pos];
            let method_name = func_path[dot_pos + 1..].to_string();
            let obj        = self.get_value(obj_name)?;
            return match obj {
                JpxValue::NativeObject(arc_obj) => {
                    // BUG FIX: unwrap() on lock() can panic if mutex is poisoned
                    match arc_obj.lock() {
                        Ok(mut guard) => guard.call_method(&method_name, args).map(Some),
                        Err(_)        => Err(JpxError::Runtime("Mutex poisoned".to_string())),
                    }
                }
                JpxValue::Object(ref m) => {
                    if let Some(func_val) = m.get(&method_name).cloned() {
                        match func_val {
                            JpxValue::NativeFunction(f) => f(args).map(Some),
                            JpxValue::Function(f) => {
                                self.call_function(&f.clone(), args).map(Some)
                            }
                            _ => Err(JpxError::Type(format!(
                                "{}.{} is not callable", obj_name, method_name
                            ))),
                        }
                    } else {
                        Ok(Some(JpxValue::Null))
                    }
                }
                _ => Err(JpxError::Type(format!(
                    "Cannot call method '{}' on {}", method_name, obj.type_name()
                ))),
            };
        }

        // Plain function call
        let func_val = self.env.get(func_path)
            .or_else(|| self.builtins.get(func_path))
            .cloned();

        if let Some(func_val) = func_val {
            return match func_val {
                JpxValue::Function(f)       => self.call_function(&f.clone(), args).map(Some),
                JpxValue::NativeFunction(f) => f(args).map(Some),
                _ => Err(JpxError::Type(format!("{} is not callable", func_path))),
            };
        }
        if let Some(f) = self.functions.get(func_path).cloned() {
            return self.call_function(&f, args).map(Some);
        }
        Ok(None)
    }

    // ─────────────────────────────────────────────────────────────
    // Assignment: var = value  (but not == != <= >=)
    // ─────────────────────────────────────────────────────────────
    fn try_eval_assignment(&mut self, expr: &str) -> Result<Option<JpxValue>, JpxError> {
        if expr.contains("==") || expr.contains("!=") || expr.contains("<=") || expr.contains(">=") {
            return Ok(None);
        }
        if expr.starts_with("if ") || expr.starts_with("while ") || expr.starts_with("for ") {
            return Ok(None);
        }
        let eq_pos = match self.find_assignment_eq(expr) {
            Some(p) => p,
            None    => return Ok(None),
        };
        let var_name = expr[..eq_pos].trim();
        let val_expr = expr[eq_pos + 1..].trim();

        if var_name.is_empty() { return Ok(None); }
        if !var_name.chars().next().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false) {
            return Ok(None);
        }

        // Compound: var+ = x  →  var = var + x
        if var_name.ends_with('+') {
            let vn = var_name.trim_end_matches('+').trim().to_string();
            let old = self.get_value(&vn).unwrap_or(JpxValue::Int(0));
            let new_val = self.eval_expression(val_expr)?;
            let result = self.apply_add(old, new_val, &vn)?;
            return Ok(Some(result));
        }
        if var_name.ends_with('-') {
            let vn = var_name.trim_end_matches('-').trim().to_string();
            let old = self.get_value(&vn).unwrap_or(JpxValue::Int(0));
            let new_val = self.eval_expression(val_expr)?;
            let result = self.apply_subtract(old, new_val)?;
            self.env.insert(vn, result.clone());
            return Ok(Some(result));
        }

        let value = self.eval_expression(val_expr)?;
        self.env.insert(var_name.to_string(), value.clone());
        Ok(Some(value))
    }

    fn apply_add(&mut self, old: JpxValue, new_val: JpxValue, var_name: &str) -> Result<JpxValue, JpxError> {
        let result = match (&old, &new_val) {
            (JpxValue::Int(a),   JpxValue::Int(b))   => JpxValue::Int(a.saturating_add(*b)),
            (JpxValue::Float(a), JpxValue::Float(b)) => JpxValue::Float(a + b),
            (JpxValue::Int(a),   JpxValue::Float(b)) => JpxValue::Float(*a as f64 + b),
            (JpxValue::Float(a), JpxValue::Int(b))   => JpxValue::Float(a + *b as f64),
            (JpxValue::String(a), _)                  => JpxValue::String(format!("{}{}", a, new_val)),
            _                                          => JpxValue::String(format!("{}{}", old, new_val)),
        };
        self.env.insert(var_name.to_string(), result.clone());
        Ok(result)
    }

    fn apply_subtract(&self, old: JpxValue, new_val: JpxValue) -> Result<JpxValue, JpxError> {
        let a = old.to_f64().unwrap_or(0.0);
        let b = new_val.to_f64().unwrap_or(0.0);
        let r = a - b;
        if r.fract() == 0.0 { Ok(JpxValue::Int(r as i64)) } else { Ok(JpxValue::Float(r)) }
    }

    /// Find '=' that is not part of ==, !=, <=, >=
    fn find_assignment_eq(&self, expr: &str) -> Option<usize> {
        let bytes = expr.as_bytes();
        let mut in_string = false;
        let mut quote_char = b'"';
        let mut depth: i32 = 0;
        for i in 0..bytes.len() {
            let b = bytes[i];
            if in_string {
                if b == quote_char && (i == 0 || bytes[i - 1] != b'\\') { in_string = false; }
                continue;
            }
            if b == b'"' || b == b'\'' { in_string = true; quote_char = b; continue; }
            if b == b'(' || b == b'[' || b == b'{' { depth += 1; continue; }
            if b == b')' || b == b']' || b == b'}' { depth -= 1; continue; }
            if b == b'=' && depth == 0 {
                let prev = if i > 0 { bytes[i - 1] } else { b' ' };
                let next = if i + 1 < bytes.len() { bytes[i + 1] } else { b' ' };
                if next == b'=' || prev == b'!' || prev == b'<' || prev == b'>' || prev == b'=' {
                    continue;
                }
                return Some(i);
            }
        }
        None
    }

    // ─────────────────────────────────────────────────────────────
    // Comparison
    // ─────────────────────────────────────────────────────────────
    fn try_eval_comparison(&mut self, expr: &str, op: &str) -> Result<Option<JpxValue>, JpxError> {
        let pos = match self.find_op_pos_outside_strings(expr, op) {
            Some(p) => p,
            None    => return Ok(None),
        };
        let left  = self.eval_expression(expr[..pos].trim())?;
        let right = self.eval_expression(expr[pos + op.len()..].trim())?;
        let result = match op {
            "==" => left == right,
            "!=" => left != right,
            "<"  => left.partial_cmp(&right).map(|o| o.is_lt()).unwrap_or(false),
            ">"  => left.partial_cmp(&right).map(|o| o.is_gt()).unwrap_or(false),
            "<=" => left.partial_cmp(&right).map(|o| o.is_le()).unwrap_or(false),
            ">=" => left.partial_cmp(&right).map(|o| o.is_ge()).unwrap_or(false),
            _    => false,
        };
        Ok(Some(JpxValue::Bool(result)))
    }

    // ─────────────────────────────────────────────────────────────
    // Addition / string concatenation
    // ─────────────────────────────────────────────────────────────
    fn try_eval_addition(&mut self, expr: &str) -> Result<Option<JpxValue>, JpxError> {
        let parts = self.split_by_op_outside_strings(expr, '+')?;
        if parts.len() <= 1 { return Ok(None); }

        let mut result_str: Option<String> = None;
        let mut result_num: Option<f64>    = None;
        let mut is_float = false;

        for part in &parts {
            let val = self.eval_expression(part.trim())?;
            match &val {
                JpxValue::String(s) => {
                    let current = result_str.get_or_insert_with(|| {
                        result_num
                            .map(|n| if is_float { format!("{}", n) } else { format!("{}", n as i64) })
                            .unwrap_or_default()
                    });
                    current.push_str(s);
                    result_num = None;
                }
                _ => {
                    if let Some(ref mut s) = result_str {
                        s.push_str(&val.to_string());
                    } else if let Some(n) = val.to_f64() {
                        if val.type_name() == "float" { is_float = true; }
                        result_num = Some(result_num.unwrap_or(0.0) + n);
                    } else {
                        let s = result_str.get_or_insert_with(String::new);
                        s.push_str(&val.to_string());
                    }
                }
            }
        }

        if let Some(s) = result_str { return Ok(Some(JpxValue::String(s))); }
        if let Some(n) = result_num {
            return if is_float { Ok(Some(JpxValue::Float(n))) }
                   else        { Ok(Some(JpxValue::Int(n as i64))) };
        }
        Ok(None)
    }

    // ─────────────────────────────────────────────────────────────
    // Split by operator char outside strings/parens
    // ─────────────────────────────────────────────────────────────
    fn split_by_op_outside_strings(&self, expr: &str, op: char) -> Result<Vec<String>, JpxError> {
        let mut parts   = Vec::new();
        let mut current = String::new();
        let mut in_str  = false;
        let mut qchar   = '"';
        let mut depth: i32 = 0;
        for ch in expr.chars() {
            if in_str {
                current.push(ch);
                if ch == qchar { in_str = false; }
            } else if ch == '"' || ch == '\'' {
                in_str = true; qchar = ch; current.push(ch);
            } else if ch == '(' || ch == '[' || ch == '{' { depth += 1; current.push(ch); }
            else if ch == ')' || ch == ']' || ch == '}' { depth -= 1; current.push(ch); }
            else if ch == op && depth == 0 {
                parts.push(current.clone()); current = String::new();
            } else { current.push(ch); }
        }
        if !current.is_empty() { parts.push(current); }
        Ok(parts)
    }

    // ─────────────────────────────────────────────────────────────
    // Subtraction (right-to-left scan to handle a-b-c correctly)
    // ─────────────────────────────────────────────────────────────
    fn try_eval_subtraction(&mut self, expr: &str) -> Result<Option<JpxValue>, JpxError> {
        let bytes = expr.as_bytes();
        let mut in_str = false;
        let mut depth: i32 = 0;
        let mut best: Option<usize> = None;

        for i in 0..bytes.len() {
            let b = bytes[i];
            if in_str {
                if b == b'"' || b == b'\'' { in_str = false; }
                continue;
            }
            if b == b'"' || b == b'\'' { in_str = true; continue; }
            if b == b'(' || b == b'[' || b == b'{' { depth += 1; continue; }
            if b == b')' || b == b']' || b == b'}' { depth -= 1; continue; }
            if b == b'-' && depth == 0 && i > 0 {
                let prev = bytes[i - 1];
                // skip if previous char is an operator (unary minus)
                if b"+*/([{= ,".contains(&prev) { continue; }
                best = Some(i); // keep the LAST valid minus (left-associative)
            }
        }

        if let Some(pos) = best {
            // Safe: '-' is always 1 byte
            let left  = expr[..pos].trim();
            let right = expr[pos + 1..].trim();
            if !left.is_empty() && !right.is_empty() {
                let lv = self.eval_expression(left)?;
                let rv = self.eval_expression(right)?;
                let a  = lv.to_f64().ok_or_else(|| JpxError::Type(format!("Cannot subtract '{}'", left)))?;
                let b  = rv.to_f64().ok_or_else(|| JpxError::Type(format!("Cannot subtract '{}'", right)))?;
                let r  = a - b;
                return if r.fract() == 0.0 { Ok(Some(JpxValue::Int(r as i64))) }
                       else                { Ok(Some(JpxValue::Float(r))) };
            }
        }
        Ok(None)
    }

    // ─────────────────────────────────────────────────────────────
    // Multiplication / Division
    // ─────────────────────────────────────────────────────────────
    fn try_eval_mul_div(&mut self, expr: &str) -> Result<Option<JpxValue>, JpxError> {
        let bytes = expr.as_bytes();
        let mut in_str = false;
        let mut depth: i32 = 0;
        let mut best: Option<(usize, char)> = None;

        for i in 0..bytes.len() {
            let b = bytes[i];
            if in_str { if b == b'"' || b == b'\'' { in_str = false; } continue; }
            if b == b'"' || b == b'\'' { in_str = true; continue; }
            if b == b'(' || b == b'[' || b == b'{' { depth += 1; continue; }
            if b == b')' || b == b']' || b == b'}' { depth -= 1; continue; }
            if (b == b'*' || b == b'/') && depth == 0 {
                best = Some((i, b as char));
            }
        }

        if let Some((pos, op)) = best {
            let left  = expr[..pos].trim();
            let right = expr[pos + 1..].trim();
            if !left.is_empty() && !right.is_empty() {
                let lv = self.eval_expression(left)?;
                let rv = self.eval_expression(right)?;
                let a  = lv.to_f64().ok_or_else(|| JpxError::Type(format!("Not a number: '{}'", left)))?;
                let b  = rv.to_f64().ok_or_else(|| JpxError::Type(format!("Not a number: '{}'", right)))?;
                let r  = if op == '/' {
                    if b == 0.0 { return Err(JpxError::Type("Division by zero".to_string())); }
                    a / b
                } else { a * b };
                return if r.fract() == 0.0 { Ok(Some(JpxValue::Int(r as i64))) }
                       else                { Ok(Some(JpxValue::Float(r))) };
            }
        }
        Ok(None)
    }

    // ─────────────────────────────────────────────────────────────
    // Modulo
    // ─────────────────────────────────────────────────────────────
    fn try_eval_modulo(&mut self, expr: &str) -> Result<Option<JpxValue>, JpxError> {
        // find % outside strings
        if let Some(pos) = self.find_op_pos_outside_strings(expr, "%") {
            let left  = expr[..pos].trim();
            let right = expr[pos + 1..].trim();
            if !left.is_empty() && !right.is_empty() {
                let lv = self.eval_expression(left)?;
                let rv = self.eval_expression(right)?;
                if let (Some(a), Some(b)) = (lv.to_i64(), rv.to_i64()) {
                    if b == 0 { return Err(JpxError::Type("Modulo by zero".to_string())); }
                    return Ok(Some(JpxValue::Int(a % b)));
                }
            }
        }
        Ok(None)
    }

    /// True if expr is fully wrapped in matching parens
    fn matching_paren(&self, expr: &str) -> bool {
        if !expr.starts_with('(') || !expr.ends_with(')') { return false; }
        let mut depth: i32 = 0;
        for (i, ch) in expr.char_indices() {
            if ch == '('      { depth += 1; }
            else if ch == ')' {
                depth -= 1;
                if depth == 0 && i == expr.len() - 1 { return true; }
                if depth == 0 { return false; }
            }
        }
        false
    }

    /// Evaluate print content (handles + concatenation)
    fn eval_print_content(&mut self, content: &str) -> Result<String, JpxError> {
        let parts = self.split_by_op_outside_strings(content, '+')?;
        if parts.len() > 1 {
            let mut result = String::new();
            for part in parts {
                let part = part.trim();
                if part.starts_with('"') && part.ends_with('"') && part.len() >= 2 {
                    result.push_str(&self.process_string(&part[1..part.len() - 1]));
                } else {
                    let val = self.eval_expression(part)?;
                    result.push_str(&val.to_string());
                }
            }
            Ok(result)
        } else {
            let val = self.eval_expression(content)?;
            Ok(val.to_string())
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Argument & collection parsing
    // ─────────────────────────────────────────────────────────────
    pub fn parse_args(&mut self, args_str: &str) -> Result<Vec<JpxValue>, JpxError> {
        if args_str.trim().is_empty() { return Ok(vec![]); }
        let parts = self.split_args(args_str);
        let mut args = Vec::new();
        for part in parts {
            let p = part.trim();
            if !p.is_empty() { args.push(self.eval_expression(p)?); }
        }
        Ok(args)
    }

    fn split_args(&self, s: &str) -> Vec<String> {
        let mut parts   = Vec::new();
        let mut current = String::new();
        let mut depth: i32 = 0;
        let mut in_str  = false;
        let mut qchar   = '"';
        for ch in s.chars() {
            if in_str {
                current.push(ch);
                if ch == qchar { in_str = false; }
            } else if ch == '"' || ch == '\'' {
                in_str = true; qchar = ch; current.push(ch);
            } else if ch == '(' || ch == '[' || ch == '{' { depth += 1; current.push(ch); }
            else if ch == ')' || ch == ']' || ch == '}' { depth -= 1; current.push(ch); }
            else if ch == ',' && depth == 0 { parts.push(current.clone()); current = String::new(); }
            else { current.push(ch); }
        }
        if !current.trim().is_empty() { parts.push(current); }
        parts
    }

    pub fn parse_list(&mut self, expr: &str) -> Result<JpxValue, JpxError> {
        // Safety: '[' and ']' are both 1 byte
        let inner = &expr[1..expr.len() - 1];
        if inner.trim().is_empty() { return Ok(JpxValue::List(vec![])); }
        let parts = self.split_args(inner);
        let mut items = Vec::new();
        for part in parts {
            let p = part.trim();
            if !p.is_empty() { items.push(self.eval_expression(p)?); }
        }
        Ok(JpxValue::List(items))
    }

    pub fn parse_object(&mut self, expr: &str) -> Result<JpxValue, JpxError> {
        let inner = expr[1..expr.len() - 1].trim();
        if inner.is_empty() { return Ok(JpxValue::Object(HashMap::new())); }
        let mut obj   = HashMap::new();
        let pairs     = self.split_by_comma_deep(inner);
        for pair in pairs {
            let pair = pair.trim();
            if pair.is_empty() { continue; }
            if let Some(colon_pos) = self.find_colon_outside_strings(pair) {
                let key     = pair[..colon_pos].trim().trim_matches('"').to_string();
                let val_str = pair[colon_pos + 1..].trim();
                let val     = self.eval_expression(val_str)?;
                obj.insert(key, val);
            }
        }
        Ok(JpxValue::Object(obj))
    }

    fn split_by_comma_deep(&self, s: &str) -> Vec<String> {
        let mut parts   = Vec::new();
        let mut current = String::new();
        let mut depth: i32 = 0;
        let mut in_str  = false;
        let mut qchar   = '"';
        for ch in s.chars() {
            if in_str { current.push(ch); if ch == qchar { in_str = false; } }
            else if ch == '"' || ch == '\'' { in_str = true; qchar = ch; current.push(ch); }
            else if ch == '(' || ch == '[' || ch == '{' { depth += 1; current.push(ch); }
            else if ch == ')' || ch == ']' || ch == '}' { depth -= 1; current.push(ch); }
            else if ch == ',' && depth == 0 { parts.push(current.clone()); current = String::new(); }
            else { current.push(ch); }
        }
        if !current.trim().is_empty() { parts.push(current); }
        parts
    }

    fn find_colon_outside_strings(&self, s: &str) -> Option<usize> {
        let mut in_str = false;
        let mut qchar  = '"';
        let mut depth: i32 = 0;
        for (i, ch) in s.char_indices() {
            if in_str { if ch == qchar { in_str = false; } continue; }
            if ch == '"' || ch == '\'' { in_str = true; qchar = ch; continue; }
            if ch == '(' || ch == '[' || ch == '{' { depth += 1; continue; }
            if ch == ')' || ch == ']' || ch == '}' { depth -= 1; continue; }
            if ch == ':' && depth == 0 { return Some(i); }
        }
        None
    }
}
