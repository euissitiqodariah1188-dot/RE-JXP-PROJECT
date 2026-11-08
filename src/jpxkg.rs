// jpxkg.rs - JPX Package Manager
// Rewritten from Python JpXkg.py to Rust
#![allow(dead_code, unused_imports, unused_variables)]

use std::path::{Path, PathBuf};
use std::io::{self, Write};
use serde_json::Value;
use sha2::{Sha256, Digest};

const VERSION: &str = "1.0.0";

fn root_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn lib_dir() -> PathBuf { root_dir().join("lib") }
fn index_path() -> PathBuf { root_dir().join("index.json") }

fn ensure_lib() { std::fs::create_dir_all(lib_dir()).ok(); }

fn sha256_file(path: &Path) -> Option<String> {
    let data = std::fs::read(path).ok()?;
    let mut h = Sha256::new();
    h.update(&data);
    Some(format!("{:x}", h.finalize()))
}

fn load_index() -> Option<Value> {
    let idx_path = index_path();
    if !idx_path.exists() {
        eprintln!("❌ index.json not found in {:?}", root_dir());
        return None;
    }
    match std::fs::read_to_string(&idx_path) {
        Ok(s) => serde_json::from_str(&s).ok().or_else(|| {
            eprintln!("❌ Failed to parse index.json"); None
        }),
        Err(e) => { eprintln!("❌ Failed to load index.json: {}", e); None }
    }
}

fn prompt_yes_no(question: &str) -> bool {
    print!("{}", question);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let lower = input.trim().to_lowercase();
    lower == "y" || lower == "yes"
}

fn install(pkg_name: &str) -> bool {
    ensure_lib();
    let idx = match load_index() { Some(v) => v, None => return false };

    let modules = match idx.get("modules").and_then(|m| m.as_array()) {
        Some(m) => m.clone(),
        None => { eprintln!("❌ No modules found in index.json"); return false; }
    };

    let pkg = match modules.iter().find(|m| m.get("name").and_then(|n| n.as_str()) == Some(pkg_name)) {
        Some(p) => p.clone(),
        None => {
            eprintln!("❌ Package '{}' not found in index", pkg_name);
            let available: Vec<&str> = modules.iter()
                .filter_map(|m| m.get("name").and_then(|n| n.as_str())).collect();
            if !available.is_empty() { println!("📦 Available: {}", available.join(", ")); }
            return false;
        }
    };

    let url = match pkg.get("download_url").and_then(|u| u.as_str()) {
        Some(u) => u.to_string(),
        None => { eprintln!("❌ No download URL for {}", pkg_name); return false; }
    };

    let install_path = pkg.get("install_path").and_then(|p| p.as_str())
        .unwrap_or(&format!("lib/{}.jpx", pkg_name)).to_string();

    let dest = root_dir().join(&install_path);
    if let Some(parent) = dest.parent() { std::fs::create_dir_all(parent).ok(); }

    if dest.exists() {
        eprintln!("⚠️ {:?} already exists", dest);
        if !prompt_yes_no("Overwrite? [y/N] ") {
            println!("📦 Installation cancelled"); return false;
        }
    }

    let tmp = dest.with_extension("tmp");
    println!("📦 Downloading {}...", pkg_name);

    let client = reqwest::blocking::Client::builder()
        .user_agent("JPX-Installer/1.0").build().unwrap_or_default();

    match client.get(&url).send() {
        Ok(resp) => {
            let total = resp.content_length().unwrap_or(0);
            let content = match resp.bytes() {
                Ok(b) => b,
                Err(e) => { eprintln!("❌ Download failed: {}", e); return false; }
            };
            if total > 0 { println!("  Progress: 100%"); }
            if std::fs::write(&tmp, &content).is_err() {
                eprintln!("❌ Write error"); return false;
            }
        }
        Err(e) => { eprintln!("❌ Download failed: {}", e); return false; }
    }

    // SHA256 verification
    if let Some(exp_sha) = pkg.get("sha256").and_then(|s| s.as_str()) {
        println!("  Verifying SHA256...");
        if let Some(actual) = sha256_file(&tmp) {
            if actual != exp_sha {
                eprintln!("❌ SHA256 mismatch\n  Expected: {}\n  Actual:   {}", exp_sha, actual);
                if !prompt_yes_no("Continue anyway? [y/N] ") {
                    std::fs::remove_file(&tmp).ok(); return false;
                }
                println!("⚠️ Continuing with mismatched checksum");
            } else { println!("✅ SHA256 OK"); }
        }
    }

    if let Err(e) = std::fs::rename(&tmp, &dest) {
        eprintln!("❌ Failed to move file: {}", e); return false;
    }

    let size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
    let size_str = if size < 1024*1024 { format!("{:.1} KB", size as f64 / 1024.0) }
                   else { format!("{:.1} MB", size as f64 / (1024.0*1024.0)) };
    println!("✅ Installed {} to {} ({})", pkg_name, install_path, size_str);

    if let Some(deps) = pkg.get("dependencies").and_then(|d| d.as_array()) {
        if !deps.is_empty() {
            println!("⚠️ Dependencies:");
            for dep in deps { if let Some(d) = dep.as_str() { println!("  • {}", d); } }
        }
    }
    true
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        println!("JPX Package Manager v{}", VERSION);
        println!("Usage: jpxkg install <package>\n       jpxkg -v, --version");
        return;
    }

    if args.len() == 2 && (args[1] == "-v" || args[1] == "--version") {
        println!("jpxkg version {}", VERSION); return;
    }

    if args[1] != "install" {
        eprintln!("❌ Unknown command: {}\nUsage: jpxkg install <package>", args[1]); return;
    }

    if args.len() < 3 {
        eprintln!("❌ Please specify a package\nUsage: jpxkg install <package>"); return;
    }

    install(&args[2]);
}
