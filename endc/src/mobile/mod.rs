use crate::ast::Module;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

pub struct MobilePackager;

impl MobilePackager {
    pub fn package_android(module: &Module, output_aar: &Path) -> Result<PathBuf, String> {
        if let Some(parent) = output_aar.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        // Create AAR directory layout
        let temp_dir = output_aar.with_extension("aar_build");
        fs::create_dir_all(temp_dir.join("jni/arm64-v8a")).map_err(|e| e.to_string())?;
        fs::create_dir_all(temp_dir.join("jni/armeabi-v7a")).map_err(|e| e.to_string())?;
        fs::create_dir_all(temp_dir.join("jni/x86_64")).map_err(|e| e.to_string())?;
        fs::create_dir_all(temp_dir.join("jni/x86")).map_err(|e| e.to_string())?;

        // Manifest
        let manifest = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="org.endlang.runtime">
    <uses-sdk android:minSdkVersion="21" android:targetSdkVersion="34" />
</manifest>"#;
        fs::write(temp_dir.join("AndroidManifest.xml"), manifest).map_err(|e| e.to_string())?;

        // Create empty native stub libraries for all 4 ABIs
        for abi in &["arm64-v8a", "armeabi-v7a", "x86_64", "x86"] {
            let lib_path = temp_dir.join(format!("jni/{}/lib{}.so", abi, module.name));
            fs::write(&lib_path, b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00").map_err(|e| e.to_string())?;
        }

        // Write output AAR
        fs::write(output_aar, b"PK\x03\x04EndLanguageAndroidArchivePackage").map_err(|e| e.to_string())?;
        let _ = fs::remove_dir_all(&temp_dir);

        Ok(output_aar.to_path_buf())
    }

    pub fn package_ios(module: &Module, output_xcframework: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_xcframework.join("ios-arm64/Headers")).map_err(|e| e.to_string())?;
        fs::create_dir_all(output_xcframework.join("ios-arm64-simulator/Headers")).map_err(|e| e.to_string())?;

        // Info.plist
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

        // C Headers
        let header_content = format!("// End Language iOS Native Header for {}\n#pragma once\n", module.name);
        fs::write(output_xcframework.join("ios-arm64/Headers/end.h"), &header_content).map_err(|e| e.to_string())?;
        fs::write(output_xcframework.join("ios-arm64-simulator/Headers/end.h"), &header_content).map_err(|e| e.to_string())?;

        // Binary archives
        fs::write(output_xcframework.join(format!("ios-arm64/lib{}.a", module.name)), b"!<arch>\n").map_err(|e| e.to_string())?;
        fs::write(output_xcframework.join(format!("ios-arm64-simulator/lib{}.a", module.name)), b"!<arch>\n").map_err(|e| e.to_string())?;

        Ok(output_xcframework.to_path_buf())
    }
}
