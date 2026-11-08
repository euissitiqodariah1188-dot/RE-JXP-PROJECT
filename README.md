# JPX Language - Rust Rewrite

Rewrite lengkap dari Python ke Rust dengan fungsi yang sama persis.

## Struktur Proyek

```
jpx-rust/
├── Cargo.toml              # Dependencies Rust
├── README.md
└── src/
    ├── main.rs             # Entry point: jpx (dari jpx.py)
    ├── jpxkg.rs            # Entry point: jpxkg (dari JpXkg.py)
    ├── interpreter/
    │   ├── mod.rs          # Module declarations
    │   ├── interpreter.rs  # Core interpreter (dari interpreter.py)
    │   ├── eval.rs         # Expression evaluator (dari eval.py)
    │   ├── exceptions.rs   # Exception types (dari exceptions.py)
    │   ├── value.rs        # Value type system (baru, pengganti Python dynamic typing)
    │   ├── function.rs     # Function execution (dari function.py)
    │   └── importer.rs     # Module importer (dari importer.py)
    └── library/
        ├── mod.rs          # Library module declarations
        ├── math.rs         # math.py → Rust
        ├── random.rs       # random.py → Rust
        ├── hash.rs         # hash.py → Rust
        ├── fs.rs           # fs.py → Rust
        ├── json_lib.rs     # json.py → Rust
        ├── string.rs       # string.py → Rust
        ├── time.rs         # time.py → Rust
        ├── requests.rs     # requests.py → Rust
        ├── speedreq.rs     # speedreq.py → Rust
        ├── scanner.rs      # scanner.py → Rust
        ├── env.rs          # env.py → Rust
        ├── system.rs       # system.py → Rust
        ├── color.rs        # color.py → Rust (via ANSI codes, bukan colorama)
        ├── jpxhash.rs      # JpXhash.py → Rust
        ├── regex_lib.rs    # ReGx.py → Rust
        ├── beuty.rs        # beuty.py → Rust (via ANSI, bukan Rich)
        ├── notification.rs # notification.py + subnotif.py → Rust
        ├── print_lib.rs    # print.py → Rust
        └── test_lib.rs     # test.py → Rust
```

## Cara Build

```bash
# Install Rust (jika belum)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build release
cargo build --release

# Binary hasil build ada di:
# ./target/release/jpx
# ./target/release/jpxkg
```

## Cara Pakai

```bash
# Jalankan file JPX
./jpx script.jpx

# Lihat versi
./jpx --version
./jpx -v

# Info sistem
./jpx --info

# Install paket
./jpxkg install nama-paket
./jpxkg -v
```

## Perbedaan dari Versi Python

| Aspek | Python | Rust |
|-------|--------|------|
| Tipe data | Dynamic | Enum `JpxValue` |
| Error handling | Exception | `Result<T, JpxError>` |
| Concurrency | GIL | Thread-safe Arc/Mutex |
| Memory | GC | Ownership/Borrow |
| Speed | ~1x | ~10-100x lebih cepat |
| Binary | Butuh Python | Single executable |

### Library yang berubah
- `colorama` → ANSI escape codes langsung
- `rich` (beuty) → ANSI codes langsung
- `win10toast` / `plyer` → `notify-send` (Linux) / `osascript` (macOS) + console fallback
- `pywin32` (win32sys, win32ui, JpXUi) → **Tidak diport** (Windows-only, tidak ada cross-platform equivalent)

> `win32sys.py`, `win32ui.py`, dan `JpXUi.py` tidak diinclude karena bergantung pada Windows API (`pywin32`, `ctypes.windll`). Di Rust cross-platform, library GUI seperti `winit` atau `egui` bisa digunakan sebagai pengganti, tapi itu akan butuh refactor besar.

## Dependencies

Lihat `Cargo.toml` untuk daftar lengkap. Dependensi utama:
- `regex` - Regular expressions
- `serde_json` - JSON parsing/serialization
- `sha2`, `sha1`, `md-5`, `sha3`, `blake2` - Hashing
- `base64`, `hex` - Encoding
- `rand` - Random number generation
- `chrono` - Date/time
- `reqwest` - HTTP requests
- `crc32fast` - CRC32 checksum
