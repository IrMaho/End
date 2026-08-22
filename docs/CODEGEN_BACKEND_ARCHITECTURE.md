# 🏗️ End Language — Code Generation Architecture
## Multi-Backend Infrastructure, Type Mapping & SSA Lowering

---

## 🌟 The Multi-Backend Design

The End compiler provides a decoupled multi-backend code generation framework:

```text
                              End AST Module
                                    │
                  ┌─────────────────┴─────────────────┐
                  ▼                                   ▼
          Semantic Analysis                   TypeInferencePass
       (Borrow/Region/Contract)            (Attach Inferred Types)
                  │                                   │
                  └─────────────────┬─────────────────┘
                                    │
                             CodeGenBackend
                  (Trait: compile_module, name, JIT/AOT)
                                    │
       ┌────────────────────────────┼────────────────────────────┐
       ▼                            ▼                            ▼
  C11 Backend                 LLVM Backend                Cranelift JIT
  (CBackend)                  (LlvmBackend)             (CraneliftBackend)
       │                            │                            │
   Clean C11                    LLVM IR (.ll)               CLIF & JIT VM
  GCC/Clang/MSVC               llc / opt / lld            Executable Native Code
```

---

## 🧩 Core Traits

### 1. `TypeMapper` Trait (`endc/src/codegen/type_mapper.rs`)
Defines the canonical mapping from End types to target backend types:
- `CTypeMapper`: C11 types (`int64_t`, `double`, `const char*`, `EndArena*`).
- `LlvmTypeMapper`: LLVM IR types (`i64`, `double`, `i8*`, `%struct.Name*`).
- `CraneliftTypeMapper`: Cranelift types (`types::I64`, `types::F64`, `types::F32X4`).

### 2. `CodeGenBackend` Trait (`endc/src/codegen/backend_trait.rs`)
Unified interface implemented by all backends:
```rust
pub trait CodeGenBackend {
    type Output;
    fn compile_module(&mut self, module: &Module) -> Result<Self::Output, BackendError>;
    fn name(&self) -> &'static str;
    fn supports_jit(&self) -> bool;
    fn supports_aot(&self) -> bool;
}
```
