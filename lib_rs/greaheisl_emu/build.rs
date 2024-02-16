use std::env;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap();
    println!(
        "cargo:rustc-link-search={}/../target/{}/",
        manifest_dir, profile
    );
}
