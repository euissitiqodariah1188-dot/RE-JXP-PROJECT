// main.rs - JPX Interpreter Entry Point
// Rewritten from Python jpx.py to Rust
#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

use jpx::interpreter::JpxInterpreter;
use jpx::interpreter::value::JpxValue;
use std::path::Path;
use std::sync::Arc;

const VERSION_NAME: &str = "JPX v1.0.0 - Rust Edition";

fn show_version() {
    println!("{}", VERSION_NAME);
    println!("Platform: {}", std::env::consts::OS);
    println!("Arch:     {}", std::env::consts::ARCH);
}

fn show_help() {
    println!("JPX Programming Language - Rust Edition");
    println!("Version: {}", VERSION_NAME);
    println!("\nUsage:");
    println!("  jpx <file.jpx>        Run a JPX script");
    println!("  jpx --version, -v     Show version");
    println!("  jpx --help, -h        Show this help");
    println!("  jpx --info            Show system info");
}

fn show_info() {
    println!("JPX System Information");
    println!("{}", "=".repeat(50));
    println!("Version:  {}", VERSION_NAME);
    println!("Platform: {}", std::env::consts::OS);
    println!("Arch:     {}", std::env::consts::ARCH);
}

fn run_file(filename: &str) {
    if !Path::new(filename).exists() {
        eprintln!("Error: File '{}' not found", filename);
        std::process::exit(1);
    }

    let code = match std::fs::read_to_string(filename) {
        Ok(c)  => c,
        Err(e) => {
            eprintln!("Error reading '{}': {}", filename, e);
            std::process::exit(1);
        }
    };

    let mut interp = JpxInterpreter::new();
    register_builtins(&mut interp);

    match interp.run(&code) {
        Ok(_)  => {}
        Err(e) => {
            eprintln!("RuntimeError: {}", e);
            std::process::exit(1);
        }
    }
}

fn register_builtins(interp: &mut JpxInterpreter) {
    use jpx::interpreter::value::JpxValue;

    // print
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        println!("{}", args.first().map(|v| v.to_string()).unwrap_or_default());
        Ok(JpxValue::Null)
    }));
    interp.builtins.insert("print".to_string(), f.clone());
    interp.env.insert("print".to_string(), f);

    // input
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        use std::io::Write;
        print!("{}", args.first().map(|v| v.to_string()).unwrap_or_default());
        std::io::stdout().flush().ok();
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).ok();
        Ok(JpxValue::String(line.trim_end_matches(['\n', '\r']).to_string()))
    }));
    interp.builtins.insert("input".to_string(), f.clone());
    interp.env.insert("input".to_string(), f);

    // str()
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        Ok(JpxValue::String(args.first().map(|v| v.to_string()).unwrap_or_default()))
    }));
    interp.builtins.insert("str".to_string(), f.clone());
    interp.env.insert("str".to_string(), f);

    // int()
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        Ok(JpxValue::Int(args.first().and_then(|v| v.to_i64()).unwrap_or(0)))
    }));
    interp.builtins.insert("int".to_string(), f.clone());
    interp.env.insert("int".to_string(), f);

    // float()
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        Ok(JpxValue::Float(args.first().and_then(|v| v.to_f64()).unwrap_or(0.0)))
    }));
    interp.builtins.insert("float".to_string(), f.clone());
    interp.env.insert("float".to_string(), f);

    // bool()
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        Ok(JpxValue::Bool(args.first().map(|v| v.is_truthy()).unwrap_or(false)))
    }));
    interp.builtins.insert("bool".to_string(), f.clone());
    interp.env.insert("bool".to_string(), f);

    // len()
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        Ok(JpxValue::Int(args.first().and_then(|v| v.len()).unwrap_or(0) as i64))
    }));
    interp.builtins.insert("len".to_string(), f.clone());
    interp.env.insert("len".to_string(), f);

    // type()
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        Ok(JpxValue::String(args.first().map(|v| v.type_name().to_string()).unwrap_or_else(|| "null".to_string())))
    }));
    interp.builtins.insert("type".to_string(), f.clone());
    interp.env.insert("type".to_string(), f);

    // push() — returns new list (JPX is functional for lists)
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        if let (Some(JpxValue::List(v)), Some(item)) = (args.first().cloned(), args.get(1).cloned()) {
            let mut v = v;
            v.push(item);
            return Ok(JpxValue::List(v));
        }
        Ok(JpxValue::Null)
    }));
    interp.builtins.insert("push".to_string(), f.clone());
    interp.env.insert("push".to_string(), f);

    // pop() — returns (last_element, rest_list) logically; here returns last element
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        if let Some(JpxValue::List(mut v)) = args.first().cloned() {
            return Ok(v.pop().unwrap_or(JpxValue::Null));
        }
        Ok(JpxValue::Null)
    }));
    interp.builtins.insert("pop".to_string(), f.clone());
    interp.env.insert("pop".to_string(), f);

    // range(end) or range(start, end) or range(start, end, step)
    // BUG FIX: guard against step=0 (would panic in step_by(0))
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        let (start, end, step) = match args.len() {
            0 => (0i64, 0i64, 1i64),
            1 => (0, args[0].to_i64().unwrap_or(0), 1),
            2 => (args[0].to_i64().unwrap_or(0), args[1].to_i64().unwrap_or(0), 1),
            _ => (
                args[0].to_i64().unwrap_or(0),
                args[1].to_i64().unwrap_or(0),
                args[2].to_i64().unwrap_or(1),
            ),
        };
        // Guard: step=0 would panic in step_by
        let step = if step == 0 { 1 } else { step };
        let items: Vec<JpxValue> = if step > 0 {
            (start..end).step_by(step as usize).map(JpxValue::Int).collect()
        } else {
            // negative step: count down
            let mut v = Vec::new();
            let mut i = start;
            while i > end { v.push(JpxValue::Int(i)); i += step; }
            v
        };
        Ok(JpxValue::List(items))
    }));
    interp.builtins.insert("range".to_string(), f.clone());
    interp.env.insert("range".to_string(), f);

    // abs()
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        match args.first() {
            Some(JpxValue::Int(n))   => Ok(JpxValue::Int(n.abs())),
            Some(JpxValue::Float(n)) => Ok(JpxValue::Float(n.abs())),
            _ => Ok(JpxValue::Int(0)),
        }
    }));
    interp.builtins.insert("abs".to_string(), f.clone());
    interp.env.insert("abs".to_string(), f);

    // min() / max()
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        let list = match args.first() {
            Some(JpxValue::List(v)) => v.clone(),
            _ => args.clone(),
        };
        let m = list.iter().filter_map(|v| v.to_f64()).fold(f64::INFINITY, f64::min);
        if m.fract() == 0.0 { Ok(JpxValue::Int(m as i64)) } else { Ok(JpxValue::Float(m)) }
    }));
    interp.builtins.insert("min".to_string(), f.clone());
    interp.env.insert("min".to_string(), f);

    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        let list = match args.first() {
            Some(JpxValue::List(v)) => v.clone(),
            _ => args.clone(),
        };
        let m = list.iter().filter_map(|v| v.to_f64()).fold(f64::NEG_INFINITY, f64::max);
        if m.fract() == 0.0 { Ok(JpxValue::Int(m as i64)) } else { Ok(JpxValue::Float(m)) }
    }));
    interp.builtins.insert("max".to_string(), f.clone());
    interp.env.insert("max".to_string(), f);

    // sum()
    let f = JpxValue::NativeFunction(Arc::new(|args: Vec<JpxValue>| {
        let list = match args.first() {
            Some(JpxValue::List(v)) => v.clone(),
            _ => return Ok(JpxValue::Int(0)),
        };
        let total: f64 = list.iter().filter_map(|v| v.to_f64()).sum();
        if total.fract() == 0.0 { Ok(JpxValue::Int(total as i64)) } else { Ok(JpxValue::Float(total)) }
    }));
    interp.builtins.insert("sum".to_string(), f.clone());
    interp.env.insert("sum".to_string(), f);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        show_help();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "-v" | "--version" => show_version(),
        "-h" | "--help"    => show_help(),
        "--info"           => show_info(),
        filename           => run_file(filename),
    }
}
