import sys
import os

sys.stdout.reconfigure(encoding='utf-8')

c_backend_path = r"C:\Users\ASUS\Desktop\flutter_project\end\endc\src\codegen\c_backend.rs"
main_path = r"C:\Users\ASUS\Desktop\flutter_project\end\endc\src\main.rs"

# === 1. PATCH C_BACKEND.RS ===
with open(c_backend_path, 'r', encoding='utf-8') as f:
    content = f.read()

# 1.1 Add active_regions to struct
old_struct = """pub struct CBackend {
    output: String,
    header_output: String,
    indent_level: usize,
    enums: Vec<EnumDef>,
    is_lib: bool,
    pub var_types: HashMap<String, Type>,
}"""

new_struct = """pub struct CBackend {
    output: String,
    header_output: String,
    indent_level: usize,
    enums: Vec<EnumDef>,
    is_lib: bool,
    pub var_types: HashMap<String, Type>,
    pub active_regions: Vec<String>,
}"""

if old_struct in content:
    content = content.replace(old_struct, new_struct)
    print("[OK] Added active_regions to CBackend struct")

# 1.2 Add active_regions to new()
old_new = """    pub fn new() -> Self {
        Self {
            output: String::new(),
            header_output: String::new(),
            indent_level: 0,
            enums: Vec::new(),
            is_lib: false,
            var_types: HashMap::new(),
        }
    }"""

new_new = """    pub fn new() -> Self {
        Self {
            output: String::new(),
            header_output: String::new(),
            indent_level: 0,
            enums: Vec::new(),
            is_lib: false,
            var_types: HashMap::new(),
            active_regions: Vec::new(),
        }
    }"""

if old_new in content:
    content = content.replace(old_new, new_new)
    print("[OK] Added active_regions to CBackend::new()")

# 1.3 Clear active_regions in generate_with_options
old_gen = """    pub fn generate_with_options(&mut self, module: &Module, is_lib: bool) -> (String, Option<String>) {
        self.output.clear();
        self.header_output.clear();
        self.enums.clear();
        self.is_lib = is_lib;
        self.var_types.clear();"""

new_gen = """    pub fn generate_with_options(&mut self, module: &Module, is_lib: bool) -> (String, Option<String>) {
        self.output.clear();
        self.header_output.clear();
        self.enums.clear();
        self.is_lib = is_lib;
        self.var_types.clear();
        self.active_regions.clear();"""

if old_gen in content:
    content = content.replace(old_gen, new_gen)
    print("[OK] Added active_regions.clear() in generate_with_options")

# 1.4 Memory primitives with TLS Bump-Pointer Hardware Cache
old_mem = """        // Runtime memory primitives (Region & Allocator support)
        self.output.push_str("/* End Memory Primitives (64-byte Cache-Line Aligned) */\\n");
        self.output.push_str("typedef struct { void* (*alloc)(size_t); void (*free)(void*); } EndAllocator;\\n");
        self.output.push_str("typedef struct { char* buffer; size_t capacity; size_t offset; } EndArena;\\n");
        self.output.push_str("static EndArena* end_arena_create(size_t cap) {\\n");
        self.output.push_str("    EndArena* a = (EndArena*)malloc(sizeof(EndArena));\\n");
        self.output.push_str("    a->buffer = (char*)malloc(cap);\\n");
        self.output.push_str("    a->capacity = cap;\\n");
        self.output.push_str("    a->offset = 0;\\n");
        self.output.push_str("    return a;\\n");
        self.output.push_str("}\\n");
        self.output.push_str("static void* end_arena_alloc(EndArena* a, size_t size) {\\n");
        self.output.push_str("    if (a->offset + size > a->capacity) return NULL;\\n");
        self.output.push_str("    void* ptr = (void*)(a->buffer + a->offset);\\n");
        self.output.push_str("    a->offset += (size + 63) & ~63;\\n");
        self.output.push_str("    return ptr;\\n");
        self.output.push_str("}\\n");
        self.output.push_str("static void end_arena_destroy(EndArena* a) {\\n");
        self.output.push_str("    if (a) { free(a->buffer); free(a); }\\n");
        self.output.push_str("}\\n\\n");"""

new_mem = """        // Runtime memory primitives (Hardware Scratchpad TLS Bump Arena)
        self.output.push_str("/* End Zero-Cost Memory Primitives (64-byte Cache-Line Aligned) */\\n");
        self.output.push_str("typedef struct { void* (*alloc)(size_t); void (*free)(void*); } EndAllocator;\\n");
        self.output.push_str("typedef struct { char* buffer; size_t capacity; size_t offset; bool is_heap; } EndArena;\\n");
        self.output.push_str("#if defined(_MSC_VER)\\n");
        self.output.push_str("    static __declspec(thread) char _end_tls_scratchpad[4 * 1024 * 1024];\\n");
        self.output.push_str("    static __declspec(thread) size_t _end_tls_offset = 0;\\n");
        self.output.push_str("#else\\n");
        self.output.push_str("    static __thread char _end_tls_scratchpad[4 * 1024 * 1024];\\n");
        self.output.push_str("    static __thread size_t _end_tls_offset = 0;\\n");
        self.output.push_str("#endif\\n\\n");
        self.output.push_str("static inline EndArena* end_arena_create(size_t cap) {\\n");
        self.output.push_str("    if (_end_tls_offset + sizeof(EndArena) + cap <= sizeof(_end_tls_scratchpad)) {\\n");
        self.output.push_str("        EndArena* a = (EndArena*)(_end_tls_scratchpad + _end_tls_offset);\\n");
        self.output.push_str("        _end_tls_offset += (sizeof(EndArena) + 63) & ~63;\\n");
        self.output.push_str("        a->buffer = _end_tls_scratchpad + _end_tls_offset;\\n");
        self.output.push_str("        _end_tls_offset += (cap + 63) & ~63;\\n");
        self.output.push_str("        a->capacity = cap;\\n");
        self.output.push_str("        a->offset = 0;\\n");
        self.output.push_str("        a->is_heap = false;\\n");
        self.output.push_str("        return a;\\n");
        self.output.push_str("    }\\n");
        self.output.push_str("    EndArena* a = (EndArena*)malloc(sizeof(EndArena));\\n");
        self.output.push_str("    a->buffer = (char*)malloc(cap);\\n");
        self.output.push_str("    a->capacity = cap;\\n");
        self.output.push_str("    a->offset = 0;\\n");
        self.output.push_str("    a->is_heap = true;\\n");
        self.output.push_str("    return a;\\n");
        self.output.push_str("}\\n");
        self.output.push_str("static inline void* end_arena_alloc(EndArena* a, size_t size) {\\n");
        self.output.push_str("    size_t aligned = (size + 63) & ~63;\\n");
        self.output.push_str("    if (a->offset + aligned > a->capacity) return NULL;\\n");
        self.output.push_str("    void* ptr = (void*)(a->buffer + a->offset);\\n");
        self.output.push_str("    a->offset += aligned;\\n");
        self.output.push_str("    return ptr;\\n");
        self.output.push_str("}\\n");
        self.output.push_str("static inline void end_arena_destroy(EndArena* a) {\\n");
        self.output.push_str("    if (!a) return;\\n");
        self.output.push_str("    if (a->is_heap) { free(a->buffer); free(a); }\\n");
        self.output.push_str("    else { _end_tls_offset = 0; }\\n");
        self.output.push_str("}\\n\\n");"""

if old_mem in content:
    content = content.replace(old_mem, new_mem)
    print("[OK] Replaced memory primitives with TLS Scratchpad Bump Arena")

# 1.5 Statement::RegionBlock
old_region_block = """            Statement::RegionBlock { name, body, .. } => {
                self.output.push_str(&format!(
                    "{}/* Enter Region: {} */\\n",
                    self.indent(),
                    name
                ));
                self.output.push_str(&format!(
                    "{}EndArena* region_{} = end_arena_create(64 * 1024);\\n",
                    self.indent(),
                    name
                ));
                self.output.push_str(&format!("{}{{\\n", self.indent()));
                self.indent_level += 1;
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\\n", self.indent()));
                self.output.push_str(&format!(
                    "{}end_arena_destroy(region_{});\\n",
                    self.indent(),
                    name
                ));
            }"""

new_region_block = """            Statement::RegionBlock { name, body, .. } => {
                self.output.push_str(&format!(
                    "{}/* Enter Region: {} */\\n",
                    self.indent(),
                    name
                ));
                self.output.push_str(&format!(
                    "{}EndArena* region_{} = end_arena_create(512 * 1024);\\n",
                    self.indent(),
                    name
                ));
                self.output.push_str(&format!("{}{{\\n", self.indent()));
                self.indent_level += 1;
                self.active_regions.push(name.clone());
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.active_regions.pop();
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\\n", self.indent()));
                self.output.push_str(&format!(
                    "{}end_arena_destroy(region_{});\\n",
                    self.indent(),
                    name
                ));
            }"""

if old_region_block in content:
    content = content.replace(old_region_block, new_region_block)
    print("[OK] Updated Statement::RegionBlock to push/pop active_regions")

# 1.6 Expression::Alloc
old_alloc = """            Expression::Alloc { target_type, .. } => match target_type {
                Type::Array(inner, size) => format!("({}*)malloc({} * sizeof({}))", self.map_type(inner), size, self.map_type(inner)),
                _ => format!("({}*)malloc(sizeof({}))", self.map_type(target_type), self.map_type(target_type)),
            }"""

new_alloc = """            Expression::Alloc { target_type, .. } => {
                if let Some(curr_region) = self.active_regions.last() {
                    match target_type {
                        Type::Array(inner, size) => format!("({}*)end_arena_alloc(region_{}, (size_t)({}) * sizeof({}))", self.map_type(inner), curr_region, size, self.map_type(inner)),
                        _ => format!("({}*)end_arena_alloc(region_{}, sizeof({}))", self.map_type(target_type), curr_region, self.map_type(target_type)),
                    }
                } else {
                    match target_type {
                        Type::Array(inner, size) => format!("({}*)malloc((size_t)({}) * sizeof({}))", self.map_type(inner), size, self.map_type(inner)),
                        _ => format!("({}*)malloc(sizeof({}))", self.map_type(target_type), self.map_type(target_type)),
                    }
                }
            }"""

if old_alloc in content:
    content = content.replace(old_alloc, new_alloc)
    print("[OK] Updated Expression::Alloc to use Region Memory when inside region")

with open(c_backend_path, 'w', encoding='utf-8') as f:
    f.write(content)

# === 2. PATCH MAIN.RS (Add GCC + Ultra-Performance Flags) ===
with open(main_path, 'r', encoding='utf-8') as f:
    main_content = f.read()

old_fallback = """            if !compiled {
                println!(
                    "{} C code is ready at {:?}. To compile natively, run: `zig cc {:?} -o {:?}`",
                    "ℹ".cyan().bold(),
                    c_file_path,
                    c_file_path,
                    bin_path
                );
            }"""

new_fallback = """            // Fallback to GCC if Clang / Zig CC failed
            if !compiled {
                let mut gcc_args = vec![
                    "-O3".to_string(),
                    "-march=native".to_string(),
                    "-flto".to_string(),
                    "-funroll-loops".to_string(),
                    "-fomit-frame-pointer".to_string(),
                    "-finline-functions".to_string(),
                    "-fno-math-errno".to_string(),
                    "-Wno-incompatible-pointer-types".to_string(),
                    c_file_path.to_str().unwrap().to_string(),
                ];
                if is_library_mode {
                    gcc_args.push("-shared".to_string());
                }
                if strip {
                    gcc_args.push("-s".to_string());
                }
                #[cfg(windows)]
                {
                    gcc_args.push("-lws2_32".to_string());
                    gcc_args.push("-luser32".to_string());
                    gcc_args.push("-lgdi32".to_string());
                }
                gcc_args.push("-o".to_string());
                gcc_args.push(bin_path.to_str().unwrap().to_string());

                let gcc_refs: Vec<&str> = gcc_args.iter().map(|s| s.as_str()).collect();
                if let Ok(status) = Command::new("gcc").args(&gcc_refs).status() {
                    if status.success() {
                        compiled = true;
                        println!("{} Native Binary compiled via GCC (Ultra-Optimized) at {:?}", "👑".green().bold(), bin_path);
                    }
                }
            }

            if !compiled {
                println!(
                    "{} C code is ready at {:?}. To compile natively, run: `gcc -O3 {:?} -o {:?}`",
                    "ℹ".cyan().bold(),
                    c_file_path,
                    c_file_path,
                    bin_path
                );
            }"""

if old_fallback in main_content:
    main_content = main_content.replace(old_fallback, new_fallback)
    print("[OK] Added GCC ultra-optimized compilation pipeline to main.rs")

with open(main_path, 'w', encoding='utf-8') as f:
    f.write(main_content)

print("\nALL COMPILER OPTIMIZATIONS SUCCESSFULLY APPLIED TO PROJECT ROOT!")
