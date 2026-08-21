---
name: end-language
description: Comprehensive expert guide for developing applications, web servers, 120 FPS native GUI applications, shared libraries (DLL), and high-performance algorithms in the End Programming Language.
---

# End Programming Language: Master Agent Reference & Architecture Guide

> **The End Programming Language (`end.exe` / `endc.exe`)** is an ultra-fast, zero-overhead, AI-first systems language combining:
> 1. **Modular Vibe-Coding & Polymorphism:** Zero-boilerplate module contracts (`derives`), invariant validation (`@invariant`), and cross-module bridges.
> 2. **Native Declarative UI (`EndUI`):** 120 FPS declarative GUI syntax (`@widget`), zero external latency, and interactive DevMode overlay.
> 3. **Bidirectional AI Agent Feedback Protocol:** Direct gallery mockup upload, visual pin annotations, and CLI reply bridge (`end agent list` / `end agent reply`).
> 4. **Universal Technical Passport (`end doc`):** Auto-generated OpenAPI 3.1 (`openapi.json`), AI Technical Passport (`project_passport.json`), and Swagger documentation.
> 5. **Native Flutter / Dart FFI Bridge (`end flutter`):** Zero-overhead Dart FFI bindgen for mobile/cross-platform deployment.
> 6. **3-Tier Hybrid Memory:** Bare-metal execution (Arena + Zero GC Region + Box + Rc/Arc) beating Zig/C/Rust/Go.

---

## 1. 🛑 The Golden Law of End Architecture (Where Code Lives)

To prevent developer and AI Agent confusion, always adhere to strict separation between **Source Code** and **Compiler Build Artifacts**:

```text
my_end_project/
│
├── 🌟 src/                             <=== ONLY WRITE SOURCE CODE HERE (100% .end files)
│   ├── main.end                        <--- Domain logic, structs, invariants, and entry point
│   └── ui_dashboard.end                <--- Declarative UI widgets (@widget) in End syntax
│
├── 📦 bin/                             <--- [AUTO-GENERATED] Native machine binaries (.exe, .dll)
├── 🌐 ui/                              <--- [AUTO-GENERATED] 120 FPS Web/Desktop DOM (end ui)
├── 📱 flutter_lib/                     <--- [AUTO-GENERATED] Dart FFI bridge for Flutter (end flutter)
├── 📄 docs/                            <--- [AUTO-GENERATED] OpenAPI & AI Passport (end doc)
└── 🤖 .end/agent_feedback/             <--- [PERSISTENT] DevMode pin annotations & mockup images
```

> [!IMPORTANT]
> **Never manually edit `ui/index.html`, `flutter_lib/*.dart`, or `docs/*.json`.**
> They are automatically compiled from your `.end` source files via `end ui`, `end flutter`, and `end doc`.

---

## 2. CLI Toolchain Command Reference

The compiler and toolchain are globally installed in system PATH as `end.exe` and `endc.exe`:

```bash
# 1. Compilation & Native Execution
end.exe run src/main.end                          # Execute immediately via VM Interpreter
end.exe build src/main.end -o bin/app.exe         # Compile to ultra-optimized machine code (-O3)
end.exe build src/lib.end --dll -o bin/lib.dll    # Compile to Shared Library / DLL + C header (.h)

# 2. Native Declarative UI & DevServer (EndUI)
end.exe ui src/main.end -o ui/ --dev --serve --port 8765 --open  # Launch live 120 FPS DevServer + AI Overlay
end.exe ui src/main.end -o ui/                                  # Production static HTML5/DOM build

# 3. Bidirectional AI Agent Feedback Protocol
end.exe agent list                                # List all open developer pins, tasks, and attached mockups
end.exe agent reply --id <id> --message "<msg>" --status Resolved  # AI Agent replies and marks task resolved

# 4. Universal Documentation & AI Technical Passport
end.exe doc src/main.end -o docs/                 # Generates openapi.json, project_passport.json, Swagger UI

# 5. Flutter / Dart FFI Bridge Generator
end.exe flutter bindgen src/main.end -o flutter_lib/  # Generates end_flutter_bridge.dart

# 6. Package Manager & Architecture Scaffolding
end.exe new my_project                            # Scaffold complete End project with End.toml
end.exe arch scaffold --feature auth --preset clean_architecture  # Generate Clean Architecture layers
end.exe add <package>                             # Add package to End.toml
```

---

## 3. End Language Syntax & Vibe-Coding Standards

### A. Modular Polymorphism & Invariant Validation
```rust
module financial_core;

// 1. Interface Declaration
pub interface LedgerProtocol {
    fn post_transaction(self, ref_id: str, debit: f64, credit: f64) bool;
    fn get_balance(self) f64;
}

// 2. Struct with Interface Derivation & Invariant Validation
pub struct DoubleEntryLedger derives LedgerProtocol {
    total_debits: f64,
    total_credits: f64,
    balanced: bool,

    // Invariant rule guaranteed by the compiler
    @invariant(self.total_debits == self.total_credits)
    pub fn verify_trial_balance(self) bool {
        return self.total_debits == self.total_credits;
    }
}
```

### B. Declarative UI Syntax (`@widget`)
```rust
module ui_dashboard;

import financial_core;

@widget
pub fn FinancialDashboard(ledger: &DoubleEntryLedger) -> Widget {
    return Scaffold {
        sidebar: Sidebar {
            brand: BrandHeader { title: "EndLedger", icon: "💼" },
            items: [
                NavItem { id: "view-overview", title: "📊 Executive Overview", active: true },
                NavItem { id: "view-invoices", title: "🧾 Invoicing & Tax Terminal" },
                NavItem { id: "view-ledger",   title: "📖 General Ledger & Audit" },
            ],
        },
        body: Column {
            spacing: 24,
            children: [
                TopBar {
                    title: "💼 EndLedger Financial Suite",
                    streamTicker: "LIVE STREAM (1.5s)",
                },
                Row {
                    spacing: 20,
                    children: [
                        StatCard { id: "cash_card", label: "Cash Reserve", value: "$59,155.00", accent: "cyan" },
                        StatCard { id: "profit_card", label: "Net Margin", value: "87.36%", accent: "emerald" },
                    ],
                },
                Card {
                    title: "🧾 Instant Invoice Terminal",
                    child: Form {
                        fields: [
                            SelectField { label: "Client", options: ["ACME Corp", "Apple Inc."] },
                            NumberField { label: "Price ($)", defaultValue: 1000 },
                        ],
                        submitButton: Button {
                            text: "⚡ Post to General Ledger",
                            onClick: "postInvoice()",
                        },
                    },
                },
            ],
        },
    };
}
```

---

## 4. AI Agent Workflow for Implementing UI & Developer Requests

When paired with a developer in an End project, follow this exact 4-step workflow:

1. **Step 1: Check Developer Feedback Channel**
   ```bash
   end.exe agent list
   ```
   Inspect open pins, developer instructions, and attached mockup images in `.end/agent_feedback/`.

2. **Step 2: Update Pure End Source Files (`src/*.end`)**
   Make all logic, structural, styling, and widget updates inside `src/main.end` and `src/ui_dashboard.end`.

3. **Step 3: Rebuild and Verify DevServer**
   ```bash
   end.exe ui src/main.end -o ui/ --dev
   ```

4. **Step 4: Reply to Developer Pins & Resolve Tasks**
   ```bash
   end.exe agent reply --id <fb_id> --message "Implemented multi-view routing and verified 120 FPS invariant." --status Resolved
   ```

This protocol ensures zero hallucination, zero environment confusion, and unmatched developer velocity!
