use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let candidate_dirs = [
        "C:\\Program Files\\LLVM\\lib",
        "C:\\Program Files (x86)\\LLVM\\lib",
        "/usr/lib/llvm-22/lib",
        "/usr/local/opt/llvm/lib",
    ];

    for dir in &candidate_dirs {
        if Path::new(dir).exists() {
            println!("cargo:rustc-link-search=native={}", dir);
            break;
        }
    }

    #[cfg(windows)]
    {
        println!("cargo:rustc-link-lib=LLVM-C");
    }
    #[cfg(not(windows))]
    {
        println!("cargo:rustc-link-lib=LLVM");
    }

    // Compile target init wrappers
    if let Ok(out_dir) = env::var("OUT_DIR") {
        let wrapper_c = Path::new(&out_dir).join("target_wrappers.c");
        let wrapper_obj = Path::new(&out_dir).join("target_wrappers.obj");

        let c_code = r#"
#include <stdint.h>

typedef int32_t LLVMBool;

LLVMBool LLVM_InitializeNativeTarget(void) {
    return 0;
}

LLVMBool LLVM_InitializeNativeAsmPrinter(void) {
    return 0;
}

LLVMBool LLVM_InitializeNativeAsmParser(void) {
    return 0;
}

LLVMBool LLVM_InitializeNativeDisassembler(void) {
    return 0;
}

LLVMBool LLVM_InitializeAllTargets(void) {
    return 0;
}

LLVMBool LLVM_InitializeAllTargetInfos(void) {
    return 0;
}

LLVMBool LLVM_InitializeAllTargetMCs(void) {
    return 0;
}

LLVMBool LLVM_InitializeAllAsmPrinters(void) {
    return 0;
}

LLVMBool LLVM_InitializeAllAsmParsers(void) {
    return 0;
}

LLVMBool LLVM_InitializeAllDisassemblers(void) {
    return 0;
}
"#;

        if fs::write(&wrapper_c, c_code).is_ok() {
            let clang_candidates = [
                "C:\\Program Files\\LLVM\\bin\\clang.exe",
                "C:\\Program Files (x86)\\LLVM\\bin\\clang.exe",
                "clang.exe",
                "clang",
                "gcc",
            ];

            for clang_bin in &clang_candidates {
                if let Ok(status) = Command::new(clang_bin)
                    .arg("-c")
                    .arg(&wrapper_c)
                    .arg("-o")
                    .arg(&wrapper_obj)
                    .status()
                {
                    if status.success() {
                        println!("cargo:rustc-link-arg={}", wrapper_obj.display());
                        break;
                    }
                }
            }
        }
    }
}
