use super::runtime::emit_all_runtime_headers;
use super::state::CBackend;
use crate::ast::*;
use std::collections::{HashMap, HashSet};

impl CBackend {
    pub fn generate(&mut self, module: &Module) -> String {
        let (c_code, _) = self.generate_with_options(module, false);
        c_code
    }

    pub fn generate_with_options(&mut self, module: &Module, is_lib: bool) -> (String, Option<String>) {
        self.output.clear();
        self.header_output.clear();
        self.is_lib = is_lib;
        self.enums = module.enums.clone();
        self.function_return_types.clear();
        self.struct_fields.clear();

        for f in &module.functions {
            self.function_return_types.insert(f.name.clone(), f.return_type.clone());
        }
        for s in &module.structs {
            let mut fields = HashMap::new();
            for fld in &s.fields {
                fields.insert(fld.name.clone(), fld.field_type.clone());
            }
            self.struct_fields.insert(s.name.clone(), fields);
        }

        emit_all_runtime_headers(&mut self.output);

        // Process Imports
        for imp in &module.imports {
            match &imp.kind {
                ImportKind::C(path) => {
                    if path.starts_with('<') && path.ends_with('>') {
                        self.output.push_str(&format!("#include {}\n", path));
                    } else {
                        self.output.push_str(&format!("#include \"{}\"\n", path));
                    }
                }
                ImportKind::Zig(path) => {
                    self.output.push_str(&format!("/* Zig module: {} */\n", path));
                }
                ImportKind::Rust(path) => {
                    self.output.push_str(&format!("/* Rust crate: {} */\n", path));
                }
                _ => {}
            }
        }
        self.output.push('\n');

        // Forward declarations of Enums and Structs
        for e in &module.enums {
            self.output.push_str(&format!("typedef struct {} {};\n", e.name, e.name));
        }
        for s in &module.structs {
            self.output.push_str(&format!("typedef struct {} {};\n", s.name, s.name));
        }
        self.output.push('\n');

        // Enum Definitions (Tagged Unions)
        for e in &module.enums {
            self.output.push_str(&format!("typedef enum {{\n"));
            for v in &e.variants {
                self.output.push_str(&format!("    {}_{},\n", e.name, v.name));
            }
            self.output.push_str(&format!("}} {}_Tag;\n\n", e.name));

            self.output.push_str(&format!("struct {} {{\n", e.name));
            self.output.push_str(&format!("    {}_Tag tag;\n", e.name));
            let has_payload = e.variants.iter().any(|v| v.payload.is_some());
            if has_payload {
                self.output.push_str("    union {\n");
                for v in &e.variants {
                    if let Some(pty) = &v.payload {
                        self.output.push_str(&format!("        {} {};\n", self.map_type(pty), v.name));
                    }
                }
                self.output.push_str("    } data;\n");
            }
            self.output.push_str("};\n\n");
        }

        // Struct Definitions
        for s in &module.structs {
            self.output.push_str(&format!("struct {} {{\n", s.name));
            for f in &s.fields {
                let c_type = self.map_type(&f.field_type);
                self.output.push_str(&format!("    {} {};\n", c_type, f.name));
            }
            self.output.push_str("};\n\n");
        }

        // Event Definitions
        for stmt in &module.statements {
            if let Statement::EventDecl(ev) = stmt {
                self.output.push_str(&format!("typedef struct {} {{\n", ev.name));
                for f in &ev.fields {
                    let c_type = self.map_type(&f.field_type);
                    self.output.push_str(&format!("    {} {};\n", c_type, f.name));
                }
                if ev.fields.is_empty() {
                    self.output.push_str("    int _dummy;\n");
                }
                self.output.push_str(&format!("}} {};\n\n", ev.name));
            }
        }

        // Header generation if in Library Mode
        if is_lib {
            self.header_output.push_str("/* End Language Generated C Header File */\n");
            self.header_output.push_str("#pragma once\n\n");
            self.header_output.push_str("#include <stdint.h>\n");
            self.header_output.push_str("#include <stdbool.h>\n\n");
            self.header_output.push_str("#if defined(_WIN32) || defined(__CYGWIN__)\n");
            self.header_output.push_str("    #ifdef BUILDING_END_DLL\n");
            self.header_output.push_str("        #define END_API __declspec(dllexport)\n");
            self.header_output.push_str("    #else\n");
            self.header_output.push_str("        #define END_API __declspec(dllimport)\n");
            self.header_output.push_str("    #endif\n");
            self.header_output.push_str("#elif defined(__GNUC__) || defined(__clang__)\n");
            self.header_output.push_str("    #define END_API __attribute__((visibility(\"default\")))\n");
            self.header_output.push_str("#else\n");
            self.header_output.push_str("    #define END_API\n");
            self.header_output.push_str("#endif\n\n");
            self.header_output.push_str("#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n");

            // Structs in header
            for s in &module.structs {
                self.header_output.push_str(&format!("typedef struct {} {{\n", s.name));
                for f in &s.fields {
                    self.header_output.push_str(&format!("    {} {};\n", self.map_type(&f.field_type), f.name));
                }
                self.header_output.push_str(&format!("}} {};\n\n", s.name));
            }
        }

        // Forward declarations of functions
        for f in &module.functions {
            let is_extern = f.directives.iter().any(|d| d.name == "@extern") || (f.body.statements.is_empty() && f.name != "main");
            if is_extern {
                continue;
            }
            let ret_type = if f.name == "main" {
                "int".to_string()
            } else {
                self.map_type(&f.return_type)
            };
            let mut params_str = Vec::new();
            if f.name == "main" && !f.params.is_empty() {
                params_str.push("int argc".to_string());
                params_str.push("char** argv".to_string());
            } else {
                for p in &f.params {
                    params_str.push(format!("{} {}", self.map_type(&p.param_type), p.name));
                }
                if params_str.is_empty() {
                    params_str.push("void".to_string());
                }
            }

            let is_explicit_inline = f.directives.iter().any(|d| d.name == "@inline" || d.name == "@always_inline");
            let is_exported = (is_lib && f.is_pub) || f.directives.iter().any(|d| d.name == "@export" || d.name == "@c_export");

            // Skip forward declaration for morphic template functions
            if f.morphic_param.is_some() {
                continue;
            }
            if f.name == "main" {
                self.output.push_str(&format!("{} {}({});\n", ret_type, f.name, params_str.join(", ")));
            } else if is_exported {
                self.output.push_str(&format!("END_API {} {}({});\n", ret_type, f.name, params_str.join(", ")));
                if is_lib {
                    self.header_output.push_str(&format!("END_API {} {}({});\n", ret_type, f.name, params_str.join(", ")));
                }
            } else if is_explicit_inline {
                self.output.push_str(&format!("static inline __attribute__((always_inline)) {} {}({});\n", ret_type, f.name, params_str.join(", ")));
            } else {
                self.output.push_str(&format!("static inline {} {}({});\n", ret_type, f.name, params_str.join(", ")));
            }
        }
        self.output.push('\n');

        if is_lib {
            self.header_output.push_str("\n#ifdef __cplusplus\n}\n#endif\n");
        }

        // Process Extensions
        for ext in &module.extensions {
            for f in &ext.functions {
                self.struct_methods.entry(ext.target.clone()).or_default().insert(f.name.clone());
                let mangled_name = format!("{}_{}", ext.target, f.name);
                let ret_type = self.map_type(&f.return_type);
                let mut params_str = Vec::new();
                for (idx, p) in f.params.iter().enumerate() {
                    if idx == 0 && (p.name == "self" || p.name == "&self") {
                        params_str.push(format!("{}* self", ext.target));
                    } else {
                        params_str.push(format!("{} {}", self.map_type(&p.param_type), p.name));
                    }
                }
                if params_str.is_empty() { params_str.push("void".to_string()); }
                self.output.push_str(&format!("static inline {} {}({});\n", ret_type, mangled_name, params_str.join(", ")));
            }
        }

        // Process Modules
        for m in &module.modules {
            for f in &m.functions {
                self.module_methods.entry(m.name.clone()).or_default().insert(f.name.clone());
                let mangled_name = format!("{}_{}", m.name, f.name);
                let ret_type = self.map_type(&f.return_type);
                let mut params_str = Vec::new();
                for p in &f.params {
                    params_str.push(format!("{} {}", self.map_type(&p.param_type), p.name));
                }
                if params_str.is_empty() { params_str.push("void".to_string()); }
                self.output.push_str(&format!("static inline {} {}({});\n", ret_type, mangled_name, params_str.join(", ")));
            }
            for ov in &m.overrides {
                self.module_methods.entry(m.name.clone()).or_default().insert(ov.name.clone());
                let mangled_name = format!("{}_{}", m.name, ov.name);
                let ret_type = self.map_type(&ov.return_type);
                let mut params_str = Vec::new();
                for p in &ov.params {
                    params_str.push(format!("{} {}", self.map_type(&p.param_type), p.name));
                }
                if params_str.is_empty() { params_str.push("void".to_string()); }
                self.output.push_str(&format!("static inline {} {}({});\n", ret_type, mangled_name, params_str.join(", ")));
            }
            if let Some(parent_name) = &m.parent {
                if let Some(parent_mod) = module.modules.iter().find(|pm| pm.name == *parent_name) {
                    for pf in &parent_mod.functions {
                        self.module_methods.entry(m.name.clone()).or_default().insert(pf.name.clone());
                        let mangled_name = format!("{}_{}", m.name, pf.name);
                        let ret_type = self.map_type(&pf.return_type);
                        let mut params_str = Vec::new();
                        for p in &pf.params {
                            params_str.push(format!("{} {}", self.map_type(&p.param_type), p.name));
                        }
                        if params_str.is_empty() { params_str.push("void".to_string()); }
                        self.output.push_str(&format!("static inline {} {}({});\n", ret_type, mangled_name, params_str.join(", ")));
                    }
                }
            }
        }
        self.output.push('\n');

        // Function Bodies (morphic specializations then regular functions)
        self.gen_morphic_specializations(module);

        for f in &module.functions {
            self.gen_function(f);
        }

        // Extension Bodies
        for ext in &module.extensions {
            for f in &ext.functions {
                let mut ext_fn = f.clone();
                ext_fn.name = format!("{}_{}", ext.target, f.name);
                if let Some(first_p) = ext_fn.params.first_mut() {
                    if first_p.name == "self" || first_p.name == "&self" {
                        first_p.name = "self".to_string();
                        first_p.param_type = Type::Pointer(Box::new(Type::Custom(ext.target.clone())));
                    }
                }
                self.gen_function(&ext_fn);
            }
        }

        // Module Function Bodies & Derived Inheritance
        for m in &module.modules {
            for f in &m.functions {
                let mut mod_fn = f.clone();
                mod_fn.name = format!("{}_{}", m.name, f.name);
                self.gen_function(&mod_fn);
            }
            for ov in &m.overrides {
                let mut ov_fn = ov.clone();
                ov_fn.name = format!("{}_{}", m.name, ov.name);
                self.gen_function(&ov_fn);
            }
            if let Some(parent_name) = &m.parent {
                if let Some(parent_mod) = module.modules.iter().find(|pm| pm.name == *parent_name) {
                    for pf in &parent_mod.functions {
                        if !m.functions.iter().any(|f| f.name == pf.name) && !m.overrides.iter().any(|ov| ov.name == pf.name) {
                            let mut inherited_fn = pf.clone();
                            inherited_fn.name = format!("{}_{}", m.name, pf.name);
                            let parent_fn_call = format!("{}_{}", parent_name, pf.name);
                            let _args_call = pf.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ");
                            inherited_fn.body = Block {
                                statements: vec![
                                    Statement::Return {
                                        value: Some(Expression::Call {
                                            callee: Box::new(Expression::Ident(parent_fn_call, pf.span.clone())),
                                            args: pf.params.iter().map(|p| Expression::Ident(p.name.clone(), p.span.clone())).collect(),
                                            span: pf.span.clone(),
                                        }),
                                        span: pf.span.clone(),
                                    }
                                ],
                                span: pf.span.clone(),
                            };
                            self.gen_function(&inherited_fn);
                        }
                    }
                }
            }
        }

        // Process Top-Level Architectural Constructs & Statements
        for stmt in &module.statements {
            self.gen_statement(stmt);
        }

        (
            self.output.clone(),
            if is_lib { Some(self.header_output.clone()) } else { None },
        )
    }

    pub(crate) fn gen_function(&mut self, func: &FunctionDef) {
        let ret_type = if func.name == "main" {
            "int".to_string()
        } else {
            self.map_type(&func.return_type)
        };

        let mut params_str = Vec::new();
        if func.name == "main" && !func.params.is_empty() {
            params_str.push("int argc".to_string());
            params_str.push("char** argv".to_string());
        } else {
            for p in &func.params {
                params_str.push(format!("{} {}", self.map_type(&p.param_type), p.name));
            }
            if params_str.is_empty() {
                params_str.push("void".to_string());
            }
        }

        let fn_name = if func.name == "main" {
            "main".to_string()
        } else {
            func.name.clone()
        };

        if func.morphic_param.is_some() {
            return;
        }

        if func.directives.iter().any(|d| d.name == "@extern") || (func.body.statements.is_empty() && func.name != "main") {
            return;
        }

        let is_explicit_inline = func.directives.iter().any(|d| d.name == "@inline" || d.name == "@always_inline");
        let is_exported = (self.is_lib && func.is_pub) || func.directives.iter().any(|d| d.name == "@export" || d.name == "@c_export");
        let is_comptime = func.directives.iter().any(|d| d.name == "@comptime");

        let prefix = if func.name == "main" {
            "".to_string()
        } else if is_exported {
            "END_API ".to_string()
        } else if is_comptime {
            "static inline __attribute__((const)) ".to_string()
        } else if is_explicit_inline {
            "static inline __attribute__((always_inline)) ".to_string()
        } else {
            "static inline ".to_string()
        };

        let clean_file = func.span.file.replace('\\', "/");
        self.output.push_str(&format!("#line {} \"{}\"\n", func.span.line, clean_file));
        self.output.push_str(&format!(
            "{}{} {}({}) {{\n",
            prefix,
            ret_type,
            fn_name,
            params_str.join(", ")
        ));

        self.indent_level += 1;
        self.scope_vars = vec![HashSet::new()];
        self.var_types.clear();
        for p in &func.params {
            self.declare_c_var(&p.name, p.param_type.clone());
        }

        if func.directives.iter().any(|d| d.name == "@telemetry") {
            self.output.push_str(&format!("{}printf(\"[TELEMETRY] Executing %s\\n\", \"{}\");\n", self.indent(), func.name));
        }
        for d in &func.directives {
            if d.name == "@invariant" {
                if let Some(cond) = d.args.first() {
                    self.output.push_str(&format!("{}assert({});\n", self.indent(), cond));
                }
            }
        }

        for stmt in &func.body.statements {
            self.gen_statement(stmt);
        }
        if func.name == "main" {
            self.output.push_str(&format!("{}return 0;\n", self.indent()));
        }
        self.indent_level -= 1;
        self.output.push_str("}\n\n");
    }

    pub(crate) fn gen_block_statements(&mut self, statements: &[Statement]) {
        self.push_c_scope();
        for s in statements {
            self.gen_statement(s);
        }
        self.pop_c_scope();
    }
}
