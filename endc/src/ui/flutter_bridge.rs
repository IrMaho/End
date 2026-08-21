use crate::ast::Module;

pub struct FlutterBridgeGenerator;

impl FlutterBridgeGenerator {
    pub fn generate_dart_bridge(module: &Module) -> String {
        let mut dart = String::new();
        dart.push_str("// 🐦 Auto-Generated Flutter / Dart FFI Bridge by End Language Compiler\n");
        dart.push_str("// Zero-Overhead Compiled Native Interop for Flutter & Dart Applications\n");
        dart.push_str("// Standalone Standard Dart SDK FFI (Zero external package dependencies)\n\n");
        dart.push_str("import 'dart:ffi' as ffi;\n");
        dart.push_str("import 'dart:io' show Platform;\n\n");

        dart.push_str("/// High-Performance Native Bridge to End Language Compiled Modules\n");
        dart.push_str("class EndNativeBridge {\n");
        dart.push_str("  static final ffi.DynamicLibrary _dylib = () {\n");
        dart.push_str("    if (Platform.isWindows) return ffi.DynamicLibrary.open('end_app.dll');\n");
        dart.push_str("    if (Platform.isAndroid || Platform.isLinux) return ffi.DynamicLibrary.open('libend_app.so');\n");
        dart.push_str("    if (Platform.isIOS || Platform.isMacOS) return ffi.DynamicLibrary.process();\n");
        dart.push_str("    throw UnsupportedError('Unsupported operating system: ${Platform.operatingSystem}');\n");
        dart.push_str("  }();\n\n");

        for f in &module.functions {
            dart.push_str(&format!("  /// Exported End Routine: fn {}()\n", f.name));
            dart.push_str(&format!("  static int {}() {{\n", f.name));
            dart.push_str(&format!("    final func = _dylib.lookupFunction<ffi.Int64 Function(), int Function()>('{}');\n", f.name));
            dart.push_str("    return func();\n");
            dart.push_str("  }\n\n");
        }

        dart.push_str("  /// Generic Native Invocation Helper\n");
        dart.push_str("  static int invoke(String functionName) {\n");
        dart.push_str("    try {\n");
        dart.push_str("      final func = _dylib.lookupFunction<ffi.Int64 Function(), int Function()>(functionName);\n");
        dart.push_str("      return func();\n");
        dart.push_str("    } catch (e) {\n");
        dart.push_str("      return -1;\n");
        dart.push_str("    }\n");
        dart.push_str("  }\n");
        dart.push_str("}\n\n");

        dart.push_str("/// EndUI Reactive Controller for Flutter & Dart UI Integration\n");
        dart.push_str("class EndUIController {\n");
        dart.push_str("  final Map<String, dynamic> state = {};\n\n");
        dart.push_str("  void emit(String event, dynamic payload) {\n");
        dart.push_str("    state[event] = payload;\n");
        dart.push_str("  }\n\n");
        dart.push_str("  dynamic get(String key) => state[key];\n");
        dart.push_str("}\n");

        dart
    }
}
