# Taurine

A small experimental x86_64 Rust kernel that yet only writes text to a bootloader-provided framebuffer.

## Requirements

- Rust nightly
- `rust-src`
- `llvm-tools-preview`
- QEMU, if you want to run the image

The required Rust components are listed in each crate's `rust-toolchain.toml`.

## Build

Build the kernel:

```powershell
cargo build --manifest-path kernel/Cargo.toml --target x86_64-unknown-none
```

Build the bootable BIOS image:

```powershell
cd spawn
cargo build
```

This creates `taurine.img` in the project root.

## Run

```powershell
qemu-system-x86_64 -drive format=raw,file=taurine.img
```

## Project Layout

- `kernel/` - the `no_std` kernel
- `spawn/` - builds the BIOS disk image with `bootloader`
- `taurine.img` - generated bootable image
