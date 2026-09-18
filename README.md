# Taurine

Taurine is a minimal experimental operating system kernel written from scratch in Rust.

The project is built around a simple idea: Taurine is the low-level foundation for **Molecule** — a distributed computing system intended to connect heterogeneous machines and use them as a shared pool of computational resources.

## Vision

Molecule is intended to connect old and heterogeneous computers, potentially across different CPU architectures, and let them cooperate on computation.

The first principle is deliberately simple:

> two Taurine nodes should be able to exchange arbitrary data at a very low level.

From there, higher-level distributed functionality can be built incrementally:

```text
                    Molecule
                       │
              distributed computing
                       │
                Molecule protocol
                       │
          ┌────────────┴────────────┐
          │                         │
       Taurine                    Taurine
          │                         │
   ┌──────┴──────┐           ┌──────┴──────┐
   │ memory      │           │ memory      │
   │ storage     │           │ storage     │
   │ devices     │           │ devices     │
   │ tasks       │           │ tasks       │
   │ transport   │           │ transport   │
   └─────────────┘           └─────────────┘
```

The implementation intentionally grows from the lowest useful primitive rather than attempting to reproduce a conventional desktop operating system immediately.

## Current state

Taurine currently has:

- x86-64 boot and kernel entry
- framebuffer output
- machine ID generation
- interrupt descriptor table
- PIC initialization
- timer interrupts
- keyboard interrupts
- a small interactive shell
- command argument parsing
- low-level serial communication experiments
- QEMU master/slave node emulation

The kernel is currently **kernel-space only**. The shell is a diagnostic/interaction interface, not a userspace shell.

Filesystem and persistent storage are the next major subsystem.

## Communication

QEMU can be used to emulate two Taurine nodes connected through a serial TCP endpoint.

Start the **master** first:

```powershell
qemu-system-x86_64.exe `
  -drive format=raw,file=.\taurine.img `
  -serial tcp:127.0.0.1:4555,server=on,wait=off
```

Then start the **slave**:

```powershell
qemu-system-x86_64.exe `
  -drive format=raw,file=.\taurine.img `
  -serial tcp:127.0.0.1:4555
```

Both instances use the same kernel image. Their role is determined by the QEMU serial connection mode.

This setup is primarily a development/emulation environment. A real UART connection provides bidirectional serial communication; other physical transports can be added later.

## Architecture

The project is currently developed around x86-64, while Molecule is intended to be architecture-independent at the protocol level.

The eventual model is:

```text
              Taurine core
             /     |      \
        x86-64   AArch64   RISC-V
             \     |      /
          common wire protocol
                    │
             Molecule network
```

Architecture-specific hardware support should remain below the common communication and distributed-computing layers.

## Roadmap

The roadmap is capability-driven rather than a strict conventional OS-development checklist.

### Kernel foundations

- [x] boot into a Rust kernel
- [x] framebuffer output
- [x] IDT and interrupt handling
- [x] timer
- [x] keyboard input
- [x] interactive shell
- [x] command arguments
- [x] initial node-to-node communication
- [ ] block device abstraction
- [ ] persistent storage
- [ ] filesystem
- [ ] executable loading
- [ ] processes/tasks
- [ ] memory management
- [ ] userspace

### Molecule

- [x] exchange data between kernel instances in QEMU
- [ ] bidirectional transport abstraction
- [ ] node discovery / handshake
- [ ] framing and message protocol
- [ ] physical node-to-node communication
- [ ] heterogeneous architecture support
- [ ] distributed task execution
- [ ] resource discovery
- [ ] distributed ML workloads

The exact order may change as the kernel evolves.

## Philosophy

Taurine is intentionally small and experimental.

There is no goal of implementing every feature expected from a conventional Unix-like operating system. Subsystems should exist because they enable something the project actually needs.

If a scheduler is not needed yet, there is no reason to build one merely because operating systems normally have schedulers. If a filesystem becomes necessary for persistent state and executable loading, then the filesystem gets built.

The objective is to understand and control the entire stack — from boot and interrupts to communication and eventually distributed computation.

## Building

The kernel is built with Rust nightly.

The project contains separate `kernel` and `spawn` components. The `spawn` component produces the bootable image used during development.

Build the image with:

```powershell
cd spawn
cargo build
cd ..
```

QEMU is recommended for development and multi-node experiments.

## License

See the repository for the current license and project files.
