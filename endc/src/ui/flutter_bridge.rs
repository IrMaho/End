use crate::ast::*;

pub struct FlutterBridgeGenerator;

impl FlutterBridgeGenerator {
    pub fn generate_dart_bridge(module: &Module) -> String {
        let mut dart = String::new();
        dart.push_str("// 🐦 Auto-Generated Flutter / Dart FFI Bridge by End Language Compiler\n");
        dart.push_str("// Zero-Overhead Compiled Native Interop for Flutter & Dart Applications\n");
        dart.push_str("// Standalone Standard Dart SDK FFI (Zero external package dependencies)\n\n");
        dart.push_str("import 'dart:ffi' as ffi;\n");
        dart.push_str("import 'dart:io' show Platform;\n\n");

        for st in &module.structs {
            dart.push_str(&format!("final class {}Native extends ffi.Struct {{\n", st.name));
            for f in &st.fields {
                let ffi_type = map_type_to_dart_ffi(&f.field_type);
                if ffi_type.starts_with("ffi.") {
                    dart.push_str(&format!("  @{}()\n", ffi_type));
                }
                dart.push_str(&format!("  external {} {};\n", map_type_to_dart(&f.field_type), f.name));
            }
            dart.push_str("}\n\n");
        }

        dart.push_str("/// High-Performance Native Bridge to End Language Compiled Modules\n");
        dart.push_str("class EndNativeBridge {\n");
        dart.push_str("  static final ffi.DynamicLibrary _dylib = () {\n");
        dart.push_str("    if (Platform.isWindows) return ffi.DynamicLibrary.open('end_app.dll');\n");
        dart.push_str("    if (Platform.isAndroid || Platform.isLinux) return ffi.DynamicLibrary.open('libend_app.so');\n");
        dart.push_str("    if (Platform.isIOS || Platform.isMacOS) return ffi.DynamicLibrary.process();\n");
        dart.push_str("    throw UnsupportedError('Unsupported operating system: ${Platform.operatingSystem}');\n");
        dart.push_str("  }();\n\n");

        for f in &module.functions {
            let native_args = f.params.iter().map(|p| map_type_to_dart_ffi(&p.param_type)).collect::<Vec<_>>().join(", ");
            let native_ret = map_type_to_dart_ffi(&f.return_type);
            let dart_args = f.params.iter().map(|p| map_type_to_dart(&p.param_type)).collect::<Vec<_>>().join(", ");
            let dart_ret = map_type_to_dart(&f.return_type);

            let params_typed = f.params.iter().map(|p| format!("{} {}", map_type_to_dart(&p.param_type), p.name)).collect::<Vec<_>>().join(", ");
            let param_names = f.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ");

            dart.push_str(&format!("  /// Exported End Routine: fn {}({})\n", f.name, params_typed));
            dart.push_str(&format!("  static {} {}({}) {{\n", dart_ret, f.name, params_typed));
            dart.push_str(&format!("    final func = _dylib.lookupFunction<{} Function({}), {} Function({})>('{}');\n", native_ret, native_args, dart_ret, dart_args, f.name));
            dart.push_str(&format!("    return func({});\n", param_names));
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

fn map_type_to_dart_ffi(ty: &Type) -> String {
    match ty {
        Type::Void => "ffi.Void".to_string(),
        Type::Bool => "ffi.Bool".to_string(),
        Type::I8 => "ffi.Int8".to_string(),
        Type::I16 => "ffi.Int16".to_string(),
        Type::I32 => "ffi.Int32".to_string(),
        Type::I64 => "ffi.Int64".to_string(),
        Type::U8 => "ffi.Uint8".to_string(),
        Type::U16 => "ffi.Uint16".to_string(),
        Type::U32 => "ffi.Uint32".to_string(),
        Type::U64 => "ffi.Uint64".to_string(),
        Type::F32 => "ffi.Float".to_string(),
        Type::F64 => "ffi.Double".to_string(),
        Type::Str => "ffi.Pointer<ffi.Utf8>".to_string(),
        Type::Pointer(_) => "ffi.Pointer<ffi.Void>".to_string(),
        Type::Custom(name) => format!("{}Native", name),
        _ => "ffi.Int64".to_string(),
    }
}

fn map_type_to_dart(ty: &Type) -> String {
    match ty {
        Type::Void => "void".to_string(),
        Type::Bool => "bool".to_string(),
        Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::U8 | Type::U16 | Type::U32 | Type::U64 => "int".to_string(),
        Type::F32 | Type::F64 => "double".to_string(),
        Type::Str => "ffi.Pointer<ffi.Utf8>".to_string(),
        Type::Pointer(_) => "ffi.Pointer<ffi.Void>".to_string(),
        Type::Custom(name) => format!("{}Native", name),
        _ => "int".to_string(),
    }
}
