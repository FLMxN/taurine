use std::{env, path::Path};

fn main() {
    let kernel_path = env::var_os("CARGO_BIN_FILE_KERNEL")
        .expect("kernel artifact path is not available");
    let image_path = env::var_os("TAURINE_IMAGE_PATH")
        .unwrap_or_else(|| "../taurine.img".into());

    bootloader::BiosBoot::new(Path::new(&kernel_path))
        .create_disk_image(Path::new(&image_path))
        .expect("failed to create boot image");
}