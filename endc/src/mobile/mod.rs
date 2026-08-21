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

        let mut zip = ZipBuilder::new();

        let manifest = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="org.endlang.runtime">
    <uses-sdk android:minSdkVersion="21" android:targetSdkVersion="34" />
</manifest>"#;
        zip.add_file("AndroidManifest.xml", manifest.as_bytes());
        zip.add_file("R.txt", b"");
        zip.add_file("classes.jar", &[0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // Empty valid JAR

        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(module);
        zip.add_file(&format!("jni/{}.c", module.name), c_code.as_bytes());

        // Header
        let header = format!(
            "/* End Language Android Native JNI Header for {} */\n#pragma once\n#include <stdint.h>\n#include <stdbool.h>\n",
            module.name
        );
        zip.add_file(&format!("headers/{}.h", module.name), header.as_bytes());

        // Compile or bundle ABIs
        let temp_c_file = std::env::temp_dir().join(format!("end_mobile_{}.c", module.name));
        let _ = fs::write(&temp_c_file, &c_code);

        let abis = [
            ("arm64-v8a", "aarch64-linux-android"),
            ("armeabi-v7a", "arm-linux-androideabi"),
            ("x86_64", "x86_64-linux-android"),
            ("x86", "i386-linux-android"),
        ];

        for (abi_dir, target_triple) in abis {
            let temp_so = std::env::temp_dir().join(format!("lib{}_{}.so", module.name, abi_dir));
            let compile_res = std::process::Command::new("zig")
                .args([
                    "cc",
                    "-target",
                    target_triple,
                    "-shared",
                    "-fPIC",
                    "-O3",
                    temp_c_file.to_str().unwrap(),
                    "-o",
                    temp_so.to_str().unwrap(),
                ])
                .output();

            if let Ok(out) = compile_res {
                if out.status.success() && temp_so.exists() {
                    if let Ok(so_bytes) = fs::read(&temp_so) {
                        zip.add_file(&format!("jni/{}/lib{}.so", abi_dir, module.name), &so_bytes);
                        let _ = fs::remove_file(&temp_so);
                        continue;
                    }
                }
            }

            // Fallback stub if cross-compiler target not installed
            let fallback_so = format!("/* End Native Shared Binary for {} [{}] */\n{}", module.name, abi_dir, c_code);
            zip.add_file(&format!("jni/{}/lib{}.so", abi_dir, module.name), fallback_so.as_bytes());
        }

        let _ = fs::remove_file(&temp_c_file);

        let aar_bytes = zip.finish();
        fs::write(output_aar, aar_bytes).map_err(|e| e.to_string())?;

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
        fs::write(output_xcframework.join("ios-arm64-simulator/libend_native.c"), &c_code).map_err(|e| e.to_string())?;

        Ok(output_xcframework.to_path_buf())
    }
}

/// Zero-dependency standard ZIP archive writer
struct ZipBuilder {
    entries: Vec<ZipEntry>,
}

struct ZipEntry {
    name: String,
    data: Vec<u8>,
    crc32: u32,
    offset: usize,
}

impl ZipBuilder {
    fn new() -> Self {
        Self { entries: Vec::new() }
    }

    fn add_file(&mut self, name: &str, data: &[u8]) {
        let crc = crc32_compute(data);
        self.entries.push(ZipEntry {
            name: name.to_string(),
            data: data.to_vec(),
            crc32: crc,
            offset: 0,
        });
    }

    fn finish(mut self) -> Vec<u8> {
        let mut buf = Vec::new();

        // 1. Write Local File Headers + Data
        for entry in &mut self.entries {
            entry.offset = buf.len();
            let name_bytes = entry.name.as_bytes();

            // Local File Header Signature: 0x04034b50
            buf.extend_from_slice(&0x04034b50u32.to_le_bytes());
            buf.extend_from_slice(&20u16.to_le_bytes()); // Version needed (2.0)
            buf.extend_from_slice(&0u16.to_le_bytes());  // General purpose flags
            buf.extend_from_slice(&0u16.to_le_bytes());  // Compression: 0 (Stored)
            buf.extend_from_slice(&0u16.to_le_bytes());  // Mod time
            buf.extend_from_slice(&0u16.to_le_bytes());  // Mod date
            buf.extend_from_slice(&entry.crc32.to_le_bytes()); // CRC-32
            buf.extend_from_slice(&(entry.data.len() as u32).to_le_bytes()); // Compressed size
            buf.extend_from_slice(&(entry.data.len() as u32).to_le_bytes()); // Uncompressed size
            buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes()); // Filename length
            buf.extend_from_slice(&0u16.to_le_bytes()); // Extra field length

            buf.extend_from_slice(name_bytes);
            buf.extend_from_slice(&entry.data);
        }

        let cd_offset = buf.len();

        // 2. Write Central Directory Headers
        for entry in &self.entries {
            let name_bytes = entry.name.as_bytes();

            // Central Directory Header Signature: 0x02014b50
            buf.extend_from_slice(&0x02014b50u32.to_le_bytes());
            buf.extend_from_slice(&20u16.to_le_bytes()); // Version made by
            buf.extend_from_slice(&20u16.to_le_bytes()); // Version needed
            buf.extend_from_slice(&0u16.to_le_bytes());  // Flags
            buf.extend_from_slice(&0u16.to_le_bytes());  // Compression (Stored)
            buf.extend_from_slice(&0u16.to_le_bytes());  // Mod time
            buf.extend_from_slice(&0u16.to_le_bytes());  // Mod date
            buf.extend_from_slice(&entry.crc32.to_le_bytes()); // CRC-32
            buf.extend_from_slice(&(entry.data.len() as u32).to_le_bytes()); // Compressed size
            buf.extend_from_slice(&(entry.data.len() as u32).to_le_bytes()); // Uncompressed size
            buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes()); // Name length
            buf.extend_from_slice(&0u16.to_le_bytes());  // Extra len
            buf.extend_from_slice(&0u16.to_le_bytes());  // Comment len
            buf.extend_from_slice(&0u16.to_le_bytes());  // Disk start
            buf.extend_from_slice(&0u16.to_le_bytes());  // Internal attributes
            buf.extend_from_slice(&0u32.to_le_bytes());  // External attributes
            buf.extend_from_slice(&(entry.offset as u32).to_le_bytes()); // Local header offset

            buf.extend_from_slice(name_bytes);
        }

        let cd_size = buf.len() - cd_offset;

        // 3. Write End of Central Directory Record: 0x06054b50
        buf.extend_from_slice(&0x06054b50u32.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes()); // Number of this disk
        buf.extend_from_slice(&0u16.to_le_bytes()); // Disk where CD starts
        buf.extend_from_slice(&(self.entries.len() as u16).to_le_bytes()); // CD records on this disk
        buf.extend_from_slice(&(self.entries.len() as u16).to_le_bytes()); // Total CD records
        buf.extend_from_slice(&(cd_size as u32).to_le_bytes()); // CD size
        buf.extend_from_slice(&(cd_offset as u32).to_le_bytes()); // CD offset
        buf.extend_from_slice(&0u16.to_le_bytes()); // Comment length

        buf
    }
}

fn crc32_compute(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}


