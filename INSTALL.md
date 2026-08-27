# 🚀 End Programming Language Installation Guide

Comprehensive installation guide for **Windows, Linux, macOS, Docker, and WSL**.

---

## 1. Quick One-Line Installers

### Windows (PowerShell 5.1+)
```powershell
irm https://github.com/IrMaho/End/releases/latest/download/install.ps1 | iex
```

### Linux & macOS (Bash / Zsh)
```bash
curl -sSf https://github.com/IrMaho/End/releases/latest/download/install.sh | sh
```

---

## 2. Package Managers

### Homebrew (macOS & Linux)
```bash
brew tap IrMaho/tap
brew install end-lang
```

### WinGet (Windows 10 / 11)
```powershell
winget install EndLanguage.End
```

### Scoop (Windows)
```powershell
scoop bucket add end https://github.com/IrMaho/End
scoop install end
```

---

## 3. Manual Installation from Binaries

1. Download the archive for your operating system from the [Releases Page](https://github.com/IrMaho/End/releases):
   - **Windows:** `end-v2.0.0-windows-x64.zip`
   - **Linux:** `end-v2.0.0-linux-x64.tar.gz`
   - **macOS Apple Silicon:** `end-v2.0.0-macos-arm64.tar.gz`
   - **macOS Intel:** `end-v2.0.0-macos-x64.tar.gz`
   - **Verification Artifact:** `production_readiness_report.json`

2. Extract to your desired directory (e.g. `C:\Program Files\EndLanguage` or `/opt/end`).
3. Add the `bin/` directory to your system `PATH`.
4. Verify by running `end --version` or `end eval "100 * 5"`.

---

## 4. Docker Container

```bash
docker run -it --rm ghcr.io/irmaho/end:latest
```

Or in your `Dockerfile`:
```dockerfile
FROM ghcr.io/irmaho/end:latest AS builder
WORKDIR /app
COPY . .
RUN end build main.end -o app
CMD ["./app"]
```

---

## 5. Building from Source

Prerequisites:
- [Rust](https://rustup.rs/) (v1.75+)
- [Zig](https://ziglang.org/) (optional, for cross-compilation) or Clang

```bash
git clone https://github.com/IrMaho/End.git
cd End/endc
cargo build --release
```
The compiled binary will be in `endc/target/release/endc`.

---

## 6. LLVM Native Backend Prerequisites

To use the opt-in native LLVM backend (`endc build --backend=llvm`):
- **LLVM Toolchain**: LLVM 15+ or LLVM 22 (with `clang` and `LLVM-C` development libraries).
- **Linker**: `clang` or MinGW `gcc` available on system `PATH`.
- **Environment**: Ensure the LLVM `bin` directory (e.g. `C:\Program Files\LLVM\bin` on Windows or `/usr/lib/llvm/bin` on Linux) is included in your `PATH`.


