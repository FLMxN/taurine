# Taurine

Taurine is an experimental x86_64 Rust kernel that boots with the `bootloader` crate and renders a minimal interactive shell directly onto a bootloader-provided framebuffer. It is intentionally small and focused on low-level kernel basics rather than a complete operating system.

## What it does

- Boots in 64-bit mode with a custom `no_std` entry point
- Sets up timer and keyboard interrupt handlers
- Draws text and UI using a simple bitmap font renderer
- Tracks elapsed lifetime and keyboard input state
- Accepts a tiny command loop for commands such as `FAINT` and `MEMTOTAL`
- Produces a raw BIOS disk image that can be launched under QEMU

## Project structure

- `kernel/` - the core kernel crate
  - `src/taurine.rs` - kernel entry point and boot logic
  - `src/noradrenaline.rs` - interrupt setup and handlers
  - `src/dopamine.rs` - framebuffer writing and keyboard input handling
  - `src/nicotine.rs` - in-kernel command execution
  - `src/glucose.rs` - low-level memory helpers
- `spawn/` - build crate that packages the kernel into a bootable BIOS image
- `taurine.img` - generated bootable disk image in the project root

## Requirements

- Rust nightly toolchain
- `rust-src`
- `llvm-tools-preview`
- QEMU for running the image

The repository pins the nightly toolchain in each crate's `rust-toolchain.toml`.

## Build

Build the kernel binary:

```powershell
cargo build --manifest-path kernel/Cargo.toml --target x86_64-unknown-none
```

Generate the bootable image:

```powershell
cargo build --manifest-path spawn/Cargo.toml
```

This produces `taurine.img` in the repository root.

## Run

```powershell
qemu-system-x86_64 -drive format=raw,file=taurine.img
```

## Notes

This project is still experimental and intentionally minimal. The kernel is meant as a low-level playground for framebuffer rendering, interrupt-driven input, and early boot behavior rather than as a finished operating system.

## Useful commands

While running the kernel, the command prompt supports simple commands like:

- `MEMTOTAL` - prints the total usable memory detected at boot
- `FAINT` - triggers an intentional shutdown path that halts the system

## License

This project does not currently declare a formal license file. If you plan to reuse or distribute it, add an explicit license before publishing beyond local development.
