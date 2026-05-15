fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        cc::Build::new()
            .file("csrc/titlebar_config.m")
            .flag("-fobjc-arc")
            .compile("titlebar_config");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rerun-if-changed=csrc/titlebar_config.m");
    }
}
