fn main() {
    embed_test_manifest();
    tauri_build::build()
}

/// Test binaries don't receive tauri-build's app manifest (its linker args
/// apply to bin targets only). Without the Common-Controls v6 dependency the
/// dialog plugin's static import of TaskDialogIndirect fails to resolve and
/// every test exe dies at startup with STATUS_ENTRYPOINT_NOT_FOUND on
/// Windows. Emit tests-only linker args with a minimal manifest.
fn embed_test_manifest() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_none() {
        return;
    }
    let out_dir = match std::env::var_os("OUT_DIR") {
        Some(dir) => std::path::PathBuf::from(dir),
        None => return,
    };
    let manifest = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
    </dependentAssembly>
  </dependency>
</assembly>
"#;
    let manifest_path = out_dir.join("taxa-test.manifest");
    if std::fs::write(&manifest_path, manifest).is_err() {
        return;
    }
    println!("cargo:rerun-if-changed=build.rs");

    // MSVC targets: do NOT emit /MANIFEST* args here — tauri-build already
    // embeds a manifest, and a second MANIFEST resource with the same ID
    // fails the link with CVT1100 "duplicate resource" (seen on the windows
    // release runner). MSVC test binaries therefore lack the Common-Controls
    // dependency (local `cargo test` on an MSVC toolchain hits
    // STATUS_ENTRYPOINT_NOT_FOUND from the dialog plugin); CI runs tests on
    // Linux, and windows-gnu dev setups are covered by the windres branch.
    if std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() != "msvc" {
        // GNU: wrap the manifest in a COFF resource object via windres.
        let rc_path = out_dir.join("taxa-test.rc");
        let obj_path = out_dir.join("taxa-test-manifest.o");
        // windres treats `\` as an escape inside quoted rc strings, so use
        // forward slashes in the path it embeds.
        let rc = format!(
            "1 24 \"{}\"\n",
            manifest_path.to_string_lossy().replace('\\', "/")
        );
        if std::fs::write(&rc_path, rc).is_err() {
            return;
        }
        let ok = std::process::Command::new("windres")
            .arg(&rc_path)
            .arg("-O")
            .arg("coff")
            .arg("-o")
            .arg(&obj_path)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            println!("cargo:rustc-link-arg={}", obj_path.to_string_lossy());
        }
    }
}
