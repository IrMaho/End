# ⚡ End Language — LLVM Debug Info & WebAssembly Backend Specification
## DWARF/CodeView Debug Symbols, WASM Direct Lowering, and Browser FFI

---

## 🌟 1. LLVM DWARF & CodeView Debug Info

The End LLVM backend (`endc/src/codegen/llvm_backend.rs`) emits full DWARF v4 and CodeView metadata for source-level debugging:

```llvm
!llvm.module.flags = !{!0, !1}
!llvm.dbg.cu = !{!2}
!0 = !{i32 2, !"Dwarf Version", i32 4}
!1 = !{i32 2, !"Debug Info Version", i32 3}
!2 = distinct !DICompileUnit(language: DW_LANG_C99, file: !3, producer: "End Compiler v2.0 (LLVM Direct)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug)
!3 = !DIFile(filename: "app.end", directory: ".")
```

### CLI Debug Flag:
```bash
$ end build app.end --backend llvm --debug-info
```

---

## 🌐 2. WebAssembly (`wasm32-wasi` & `wasm32-unknown-unknown`) Backend

The End WebAssembly backend (`endc/src/codegen/wasm_backend.rs`) emits WebAssembly Text Format (`.wat`) and JavaScript/TypeScript runtime glue code for instant browser execution:

```wat
(module
  (memory (export "memory") 2)
  (import "env" "println" (func $println (param i64)))
  (func $calculate (export "calculate") (param $x i64) (result i64)
    local.get $x
    i64.const 2
    i64.mul
    return
  )
)
```

### CLI WASM Flag:
```bash
$ end build app.end --emit-wasm
✔ Generated WebAssembly WAT at "app.wat" (and JS runtime glue)
```
