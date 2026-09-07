use std::{env, path::Path};

fn main() {
    let kernel_path = env::var_os("CARGO_BIN_FILE_KERNEL")
        .expect("kernel artifact path is not available");

    bootloader::BiosBoot::new(Path::new(&kernel_path))
        .create_disk_image(Path::new("../taurine.img"))
        .expect("failed to create boot image");
}