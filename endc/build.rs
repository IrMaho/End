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

extern LLVMBool LLVMInitializeX86TargetInfo(void);
extern LLVMBool LLVMInitializeX86Target(void);
extern LLVMBool LLVMInitializeX86TargetMC(void);
extern LLVMBool LLVMInitializeX86AsmPrinter(void);
extern LLVMBool LLVMInitializeX86AsmParser(void);
extern LLVMBool LLVMInitializeX86Disassembler(void);

LLVMBool LLVM_InitializeNativeTarget(void) {
    LLVMInitializeX86TargetInfo();
    LLVMInitializeX86Target();
    LLVMInitializeX86TargetMC();
    return 0;
}

LLVMBool LLVM_InitializeNativeAsmPrinter(void) {
    LLVMInitializeX86AsmPrinter();
    return 0;
}

LLVMBool LLVM_InitializeNativeAsmParser(void) {
    LLVMInitializeX86AsmParser();
    return 0;
}

LLVMBool LLVM_InitializeNativeDisassembler(void) {
    LLVMInitializeX86Disassembler();
    return 0;
}

LLVMBool LLVM_InitializeAllTargets(void) {
    return LLVM_InitializeNativeTarget();
}

LLVMBool LLVM_InitializeAllTargetInfos(void) {
    LLVMInitializeX86TargetInfo();
    return 0;
}

LLVMBool LLVM_InitializeAllTargetMCs(void) {
    LLVMInitializeX86TargetMC();
    return 0;
}

LLVMBool LLVM_InitializeAllAsmPrinters(void) {
    return LLVM_InitializeNativeAsmPrinter();
}

LLVMBool LLVM_InitializeAllAsmParsers(void) {
    return LLVM_InitializeNativeAsmParser();
}

LLVMBool LLVM_InitializeAllDisassemblers(void) {
    return LLVM_InitializeNativeDisassembler();
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
