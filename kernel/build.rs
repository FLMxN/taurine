use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let env_path = manifest_dir.join("../spawn/.env");

    println!("cargo:rerun-if-changed={}", env_path.display());
    dotenv::from_path(&env_path).ok();

    if let Ok(version) = std::env::var("VERSION") {
        println!("cargo:rustc-env=VERSION={version}");
    }
}