# 👑 The End Programming Language: Comprehensive Developer & Enterprise Guide

> **The Complete Handbook for Building Native High-Performance Applications, Web Servers, 120 FPS Declarative UIs, Microservices, and AI Agent Systems in End.**  
> *Compiler: `endc` v2.0.0 (LLVM 18 / Cranelift / Native C11 / WebAssembly)*

---

## 1. Quick Start & Toolchain Setup

### 1.1 Verifying Installation
```bash
endc --version
# Output: endc v2.0.0 (Autonomous, Multi-Backend, 120 FPS Compiler & Real-time AI Agent Runtime)
```

### 1.2 Creating Your First Project
```bash
endc new my_app
cd my_app
```
This generates standard project structure:
```text
my_app/
├── end.config.toml        # Compiler guardrails, linter rules, architecture limits
├── Architecture.toml      # Dependency & layer firewall boundaries
├── src/
│   └── main.end          # Application entrypoint
└── bin/                   # Output binaries (.exe, .dll)
```

### 1.3 Running and Building
```bash
# 1. Instant execution via Interpreter VM
endc run src/main.end

# 2. Compile to native stripped machine binary with -O3 optimizations
endc build src/main.end -o bin/my_app.exe --strip

# 3. Compile as a Windows DLL / Linux .so shared library with C header
endc build src/main.end --dll -o bin/my_app.dll
```

---

## 2. Basic Syntax & Fundamentals

### 2.1 Hello World
```end
pub fn main() -> i32 {
    println("Hello, World from End!");
    ret 0;
}
```

### 2.2 Variables & Immutability
In End, variables are **immutable by default (`val`)**. To declare a mutable variable, use `mut` or `var`:
```end
// Immutable constant
val max_connections: i64 = 1000;
val app_name = "EndServer"; // Type inferred as str

// Mutable variable
mut active_users: i64 = 0;
active_users = active_users + 1;

// Walrus inline assignment
if (pending := get_queue_size()) > 0 {
    println("Processing items: " + pending);
}
```

### 2.3 Functions, Named Arguments & Default Parameters
```end
pub fn calculate_price(unit_price: f64, quantity: i64, discount_pct: f64 = 0.0, tax_rate: f64 = 0.09) -> f64 {
    val subtotal = unit_price * (quantity as f64);
    val discounted = subtotal * (1.0 - discount_pct);
    ret discounted * (1.0 + tax_rate);
}

pub fn main() -> i32 {
    // Calling with positional arguments
    val p1 = calculate_price(100.0, 5);
    
    // Calling with explicit named arguments
    val p2 = calculate_price(unit_price: 250.0, quantity: 2, discount_pct: 0.15);
    
    println("P1: " + p1 + ", P2: " + p2);
    ret 0;
}
```

### 2.4 Structs & Pattern Matching
```end
pub struct Customer {
    pub id: i64,
    pub name: str,
    pub tier: str,
    pub balance: f64,
}

pub enum OrderState {
    Draft,
    Pending(i64), // timestamp
    Completed(str), // transaction id
    Failed(str), // error reason
}

pub fn evaluate_order(state: OrderState) -> str {
    ret match state {
        OrderState.Draft => "Order is in draft",
        OrderState.Pending(ts) => "Order pending since " + ts,
        OrderState.Completed(tx) => "Order fulfilled: " + tx,
        OrderState.Failed(err) => "Order failed: " + err,
        _ => "Unknown order state"
    };
}
```

---

## 3. Modern Expressive Syntaxes

### 3.1 Dart-Style Cascades (`..` and `?..`)
Cascades allow chaining mutations on the same object without repeating the variable name:
```end
pub struct PaintConfig {
    pub color: str,
    pub stroke_width: i64,
    pub anti_alias: bool,
}

pub fn create_painter() -> PaintConfig {
    val painter = PaintConfig { color: "black", stroke_width: 1, anti_alias: false }
        ..color = "blue"
        ..stroke_width = 4
        ..anti_alias = true;
    ret painter;
}
```

### 3.2 List, Dict & Set Comprehensions
```end
val numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// List comprehension
val even_squares = [x * x for x in numbers if x % 2 == 0];

// Dict comprehension
val customer_map = {c.id: c.name for c in customers if c.tier == "VIP"};

// Set comprehension
val unique_categories = {item.category for item in inventory};
```

### 3.3 Spread Operators (`...` and `...?`) & Collection Control Flow
```end
val base_headers = ["Accept: application/json", "User-Agent: EndClient/2.0"];
val custom_headers = ["Authorization: Bearer token123"];

val all_headers = [
    ...base_headers,
    if is_authenticated ...custom_headers else "X-Guest: true",
    "X-Timestamp: 1700000000"
];
```

### 3.4 Null-Coalescing (`??`), Quantum Unwrap (`val!`) & Null Collapse (`~>`)
```end
// Null-coalescing default
val username = get_custom_name() ?? "Guest";

// Quantum unwrap with fallback expression
val! port = parse_port_env() ?? 8080;

// Null-collapse safe navigation pipe
val city = user ~> get_address() ~> get_city() ?? "Tehran";
```

---

## 4. Ephemeral Resource Leasing & 4-Tier Memory

End completely eliminates garbage collection pauses while maintaining mathematical memory safety through its **4-Tier Memory Engine**:

### 4.1 Ephemeral Memory Leases (`lease val`)
```end
pub fn process_large_payload(payload_size: usize) {
    // Leased buffer is allocated in JIT pool and zeroed out at scope exit with 0 ns overhead
    lease val scratch_buffer = alloc(payload_size) {
        // Read, mutate, compute
        scratch_buffer[0] = 0xFF;
    };
    // scratch_buffer is completely reclaimed here automatically
}
```

### 4.2 CPU Burst Leasing (`lease cpu`)
```end
pub fn run_intensive_simulation() {
    // Locks 8 CPU cores at realtime priority for high-intensity loop
    lease cpu(8, "realtime") {
        parallel for i in 0..<1000000 {
            compute_particle(i);
        }
    };
    // Cores are released back immediately to OS, reducing thermal power
}
```

### 4.3 Ephemeral Event Lifecycle Leasing (`lease listen`)
```end
pub fn monitor_temp_event(order_id: i64) {
    // Automatically subscribes to event while active and unsubscribes on exit
    lease listen("order_status_changed") while is_processing {
        println("Status updated!");
    };
}
```

---

## 5. Building High-Performance Web Services (`EndNexus`)

End includes a built-in enterprise async HTTP/1.1, HTTP/2, and WebSocket server capable of 100,000+ req/sec:

```end
import std.net.http;
import std.json;

pub fn main() -> i32 {
    val server = HttpServer.bind("0.0.0.0", 8080);
    println("🚀 EndNexus server listening on http://0.0.0.0:8080");

    server.router {
        get "/" => {
            ret HttpResponse.ok("Welcome to EndNexus High-Performance Backend!");
        },
        
        get "/api/health" => {
            ret HttpResponse.json({
                "status": "healthy",
                "uptime": get_uptime_ms(),
                "memory_tier": "Tier0_Ephemeral"
            });
        },
        
        post "/api/users" => {
            val body = request.body.parse_json();
            val name = body["name"] ?? "Anonymous";
            ret HttpResponse.created({ "id": 101, "name": name });
        }
    };

    server.listen();
    ret 0;
}
```

---

## 6. Building 120 FPS Native Declarative UIs (`EndUI`)

End features a native declarative UI paradigm (`@widget`) with zero external web engine overhead:

```end
import std.ui;

@widget
pub fn UserDashboard(customer: Customer) {
    Container {
        padding: 16,
        background: Color.from_hex("#1E1E2E"),
        
        Column {
            Text {
                content: "Welcome, " + customer.name,
                font_size: 24,
                color: Color.White,
                font_weight: "Bold"
            },
            
            Spacer { height: 12 },
            
            Row {
                Card {
                    title: "Balance",
                    value: "$" + customer.balance,
                    icon: "wallet",
                    color: Color.Green
                },
                Card {
                    title: "Tier Status",
                    value: customer.tier,
                    icon: "shield",
                    color: Color.Blue
                }
            },
            
            Spacer { height: 16 },
            
            Button {
                label: "Deposit Funds",
                on_click: () => {
                    navigate_to("/deposit");
                }
            }
        }
    }
}
```

Launch the UI in live DevServer mode with AI Canvas Overlay:
```bash
endc ui src/main.end --dev --serve --port 3000 --open
```

---

## 7. First-Class Operation Algebra & Resilient Pipelines

```end
// Define independent operations
operation FetchStockPrice(ticker: str) -> f64 {
    requires: ticker.len() > 0;
    guarantees: result >= 0.0;
    effects: [NetworkRead];
    emits: [PriceFetched];
    
    val price = http_get("/stocks/" + ticker).parse_f64();
    ret price;
}

operation ComputeMargin(price: f64) -> f64 {
    ret price * 1.15;
}

// Compose operations algebraically
val StockPipeline = compose {
    FetchStockPrice >> ComputeMargin;
};

// Resilient retry wrapper
val SafeFetcher = retry FetchStockPrice up to 3;

// Parallel branch computation
val MarketOverview = FetchStockPrice("AAPL") || FetchStockPrice("GOOGL") || FetchStockPrice("MSFT");
```

---

## 8. Multi-Target FFI & Cross-Platform Bindings

### 8.1 Exporting to C, Python, TypeScript and Flutter
Annotate structs and functions with `@export`:
```end
@export(C, Flutter, WASM, Python)
pub struct MathResult {
    pub sum: f64,
    pub average: f64,
    pub status: i32,
}

@export(C, Flutter, WASM, Python)
pub fn process_data_simd(data: []f32, count: i64) -> MathResult {
    // Vectorized calculation
    mut sum = 0.0;
    for x in data { sum = sum + (x as f64); }
    ret MathResult {
        sum: sum,
        average: sum / (count as f64),
        status: 0
    };
}
```

### 8.2 Generating Flutter / Dart FFI Bridge
```bash
endc flutter bindgen src/main.end -o flutter_lib/
```
Generates `flutter_lib/end_flutter_bridge.dart` with 0 manual boilerplate and 0 LSP diagnostics.

---

## 9. AI Agent Governance & Invariant Contracts

```end
// Formal architectural contract
agent SecurityAuditor {
    scope: "src/auth",
    goal: "Eliminate all plain-text secret logging",
    constraints: [
        "Passwords and JWT tokens must be typed as secret",
        "PCI-DSS compliance invariant must hold"
    ]
}

task MigrateToArgon2PasswordHash {
    owner: "Agent-CodeFixer",
    status: "Ready for Verification",
    requirement: "SEC-801",
    skills: ["CryptoArgon2", "ZeroAllocPass"],
    
    evidence {
        benchmark: "Argon2id hashing completed in 42ms with 64MB cost",
        tests: "100% tests passing in tests/security/"
    }
}
```

---

## 10. Summary Checklist for Writing End Code

1. ✅ **Immutability First:** Use `val` by default; use `mut` only when variable reassignment is required.
2. ✅ **Return Semicolons:** Terminate statements with `;`.
3. ✅ **Error Handling:** Use fallible results `!T`, quantum unwrap `val! x = expr ?? default`, or `catch err { ... }`.
4. ✅ **Zero Leaks:** Wrap high-allocation routines in `lease val buffer = alloc(...) { ... }` or `region Temp { ... }`.
5. ✅ **Clean UI:** Use `@widget` and declarative UI trees with `Container`, `Column`, `Row`, `Button`.
6. ✅ **Multi-Platform Ready:** Use `@export(C, Flutter, WASM)` for single-codebase cross-platform deployment.
