fn main() {
    let version = std::env::var("BUILD_VERSION")
        .or_else(|_| std::env::var("CARGO_PKG_VERSION"))
        .unwrap_or_else(|_| "unknown".to_string());

    println!("cargo:rustc-env=BUILD_VERSION={version}");
    println!("cargo:rerun-if-env-changed=BUILD_VERSION");
}
