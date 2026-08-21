use crate::ast::Module;

pub struct FlutterBridgeGenerator;

impl FlutterBridgeGenerator {
    pub fn generate_dart_bridge(module: &Module) -> String {
        let mut dart = String::new();
        dart.push_str("// 🐦 Auto-Generated Flutter / Dart FFI Bridge by End Language Compiler\n");
        dart.push_str("// Zero-Overhead Compiled Native Interop for Flutter & Dart Applications\n\n");
        dart.push_str("import 'dart:ffi' as ffi;\n");
        dart.push_str("import 'package:ffi/ffi.dart';\n");
        dart.push_str("import 'dart:io' show Platform;\n");
        dart.push_str("import 'package:flutter/material.dart';\n\n");

        dart.push_str("class EndNativeBridge {\n");
        dart.push_str("  static final ffi.DynamicLibrary _dylib = () {\n");
        dart.push_str("    if (Platform.isWindows) return ffi.DynamicLibrary.open('end_app.dll');\n");
        dart.push_str("    if (Platform.isAndroid || Platform.isLinux) return ffi.DynamicLibrary.open('libend_app.so');\n");
        dart.push_str("    if (Platform.isIOS || Platform.isMacOS) return ffi.DynamicLibrary.process();\n");
        dart.push_str("    throw UnsupportedError('Unsupported operating system');\n");
        dart.push_str("  }();\n\n");

        for f in &module.functions {
            dart.push_str(&format!("  // Exported End Routine: fn {}()\n", f.name));
            dart.push_str(&format!("  static int {}() {{\n", f.name));
            dart.push_str(&format!("    final func = _dylib.lookupFunction<ffi.Int64 Function(), int Function()>('{}');\n", f.name));
            dart.push_str("    return func();\n");
            dart.push_str("  }\n\n");
        }

        dart.push_str("}\n\n");

        dart.push_str("// Flutter Declarative Widget Wrapper Example\n");
        dart.push_str("class EndAppDashboard extends StatelessWidget {\n");
        dart.push_str("  const EndAppDashboard({super.key});\n\n");
        dart.push_str("  @override\n");
        dart.push_str("  Widget build(BuildContext context) {\n");
        dart.push_str("    return Scaffold(\n");
        dart.push_str("      backgroundColor: const Color(0xFF07090E),\n");
        dart.push_str("      appBar: AppBar(\n");
        dart.push_str("        title: const Text('EndUI Flutter High-Performance View'),\n");
        dart.push_str("        backgroundColor: const Color(0xFF0D121D),\n");
        dart.push_str("      ),\n");
        dart.push_str("      body: Center(\n");
        dart.push_str("        child: ElevatedButton(\n");
        dart.push_str("          onPressed: () {\n");
        if let Some(first_fn) = module.functions.first() {
            dart.push_str(&format!("            final result = EndNativeBridge.{}();\n", first_fn.name));
            dart.push_str("            ScaffoldMessenger.of(context).showSnackBar(\n");
            dart.push_str("              SnackBar(content: Text('Native End Routine Returned: $result')),\n");
            dart.push_str("            );\n");
        }
        dart.push_str("          },\n");
        dart.push_str("          child: const Text('Invoke Native End Backend'),\n");
        dart.push_str("        ),\n");
        dart.push_str("      ),\n");
        dart.push_str("    );\n");
        dart.push_str("  }\n");
        dart.push_str("}\n");

        dart
    }
}
