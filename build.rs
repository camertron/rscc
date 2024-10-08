fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");

    println!("cargo:rerun-if-changed=version.rs");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();

    let version = match std::env::var_os("RSCC_VERSION") {
        Some(ver) => ver.to_str().unwrap().to_string(),
        None => "0.0.0".to_string()
    };

    let path = std::path::Path::new(&out_dir).join("version.rs");
    let code = format!("pub fn version() -> &'static str {{ \"{}\" }}", version);
    std::fs::write(&path, code).unwrap();
}
