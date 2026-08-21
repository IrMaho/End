// 🐦 Auto-Generated Flutter / Dart FFI Bridge by End Language Compiler
// Zero-Overhead Compiled Native Interop for Flutter & Dart Applications
// Standalone Standard Dart SDK FFI (Zero external package dependencies)

import 'dart:ffi' as ffi;
import 'dart:io' show Platform;

/// High-Performance Native Bridge to End Language Compiled Modules
class EndNativeBridge {
  static final ffi.DynamicLibrary _dylib = () {
    if (Platform.isWindows) return ffi.DynamicLibrary.open('end_app.dll');
    if (Platform.isAndroid || Platform.isLinux) return ffi.DynamicLibrary.open('libend_app.so');
    if (Platform.isIOS || Platform.isMacOS) return ffi.DynamicLibrary.process();
    throw UnsupportedError('Unsupported operating system: ${Platform.operatingSystem}');
  }();

  /// Exported End Routine: fn main()
  static int main() {
    final func = _dylib.lookupFunction<ffi.Int64 Function(), int Function()>('main');
    return func();
  }

  /// Generic Native Invocation Helper
  static int invoke(String functionName) {
    try {
      final func = _dylib.lookupFunction<ffi.Int64 Function(), int Function()>(functionName);
      return func();
    } catch (e) {
      return -1;
    }
  }
}

/// EndUI Reactive Controller for Flutter & Dart UI Integration
class EndUIController {
  final Map<String, dynamic> state = {};

  void emit(String event, dynamic payload) {
    state[event] = payload;
  }

  dynamic get(String key) => state[key];
}
