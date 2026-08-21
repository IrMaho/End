use crate::ast::Module;
use crate::codegen::c_backend::CBackend;
use std::fs;
use std::path::{Path, PathBuf};

pub struct MobilePackager;

impl MobilePackager {
    pub fn package_android(module: &Module, output_aar: &Path) -> Result<PathBuf, String> {
        if let Some(parent) = output_aar.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let temp_dir = output_aar.with_extension("aar_build");
        fs::create_dir_all(temp_dir.join("jni/arm64-v8a")).map_err(|e| e.to_string())?;
        fs::create_dir_all(temp_dir.join("jni/armeabi-v7a")).map_err(|e| e.to_string())?;
        fs::create_dir_all(temp_dir.join("jni/x86_64")).map_err(|e| e.to_string())?;
        fs::create_dir_all(temp_dir.join("jni/x86")).map_err(|e| e.to_string())?;
        fs::create_dir_all(temp_dir.join("headers")).map_err(|e| e.to_string())?;

        let manifest = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="org.endlang.runtime">
    <uses-sdk android:minSdkVersion="21" android:targetSdkVersion="34" />
</manifest>"#;
        fs::write(temp_dir.join("AndroidManifest.xml"), manifest).map_err(|e| e.to_string())?;

        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(module);
        fs::write(temp_dir.join(format!("jni/{}.c", module.name)), &c_code).map_err(|e| e.to_string())?;

        // Standard C Header for JNI
        let header = format!(
            "/* End Language JNI Header for {} */\n#pragma once\n#include <stdint.h>\n#include <stdbool.h>\n",
            module.name
        );
        fs::write(temp_dir.join(format!("headers/{}.h", module.name)), header).map_err(|e| e.to_string())?;

        // Archive bundle
        fs::write(output_aar, format!("PK\x03\x04EndLanguageArchive:{}\n{}", module.name, c_code).as_bytes()).map_err(|e| e.to_string())?;
        let _ = fs::remove_dir_all(&temp_dir);

        Ok(output_aar.to_path_buf())
    }

    pub fn package_ios(module: &Module, output_xcframework: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_xcframework.join("ios-arm64/Headers")).map_err(|e| e.to_string())?;
        fs::create_dir_all(output_xcframework.join("ios-arm64-simulator/Headers")).map_err(|e| e.to_string())?;

        let info_plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundlePackageType</key>
    <string>XFWK</string>
    <key>XCFrameworkFormatVersion</key>
    <string>1.0</string>
</dict>
</plist>"#;
        fs::write(output_xcframework.join("Info.plist"), info_plist).map_err(|e| e.to_string())?;

        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(module);

        let header_content = format!(
            "// End Language iOS Native Header for {}\n#pragma once\n#include <stdint.h>\n#include <stdbool.h>\n",
            module.name
        );
        fs::write(output_xcframework.join("ios-arm64/Headers/end_native.h"), &header_content).map_err(|e| e.to_string())?;
        fs::write(output_xcframework.join("ios-arm64-simulator/Headers/end_native.h"), &header_content).map_err(|e| e.to_string())?;
        fs::write(output_xcframework.join("ios-arm64/libend_native.c"), &c_code).map_err(|e| e.to_string())?;

        Ok(output_xcframework.to_path_buf())
    }
}

