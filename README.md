# Taurine § <br />Experimental operating system kernel written from scratch in Rust for heterogenous distributed computation

## Introduction
**Taurine** is an open-source minimal experimental operating system kernel written from scratch in Rust.

The project is built around a simple idea: **Taurine** is the low-level foundation for **Molecule** — a distributed computing system intended to connect heterogeneous machines and use them as a shared pool of computational resources.

> [!IMPORTANT]
> **Taurine** is NOT tested and NOT recommended to boot on bare-metal devices (outside of VMs). But in case you do, feel free to tell us!

### License

GNU Public License v3.0

### Vision

**Molecule** is intended to connect old and heterogeneous computers, potentially across different CPU architectures, and let them cooperate on computation.

The first principle is deliberately simple:

> two **Taurine** nodes should be able to exchange arbitrary data at a very low level.

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

The implementation intentionally grows from the lowest useful primitive rather than attempting to reproduce a conventional desktop operating system.

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

The kernel is currently **kernel-space only**. The shell is a diagnostic/interaction interface, <ins>not</ins> a userspace shell.

Filesystem and persistent storage are the next major subsystem.

## Startup

QEMU can be used to emulate two Taurine nodes connected through a serial TCP endpoint. For example, on Windows host:

Wake the **master** first:

```powershell
qemu-system-x86_64.exe `
  -drive format=raw,file=.\taurine.img `
  -serial tcp:127.0.0.1:4555,server=on,wait=off
```

Then wake the **slave**:

```powershell
qemu-system-x86_64.exe `
  -drive format=raw,file=.\taurine.img `
  -serial tcp:127.0.0.1:4555
```

![Sequential boot](media/boot.gif)

Both instances use the same kernel image. Their role is determined by the QEMU serial connection mode.

QEMU keyword arguments and host OS choice is up to user/tester/developer.

> [!NOTE]
> This setup is primarily a development/emulation environment. A real UART connection provides bidirectional serial communication; other physical transports will be added later.

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
- [x] in-memory filesystem
- [ ] block device filesystem
- [ ] executable loading
- [ ] processes/tasks
- [ ] memory management
- [ ] userspace

![Interactive shell](media/shell.gif)

### Molecule

- [x] exchange data between kernel instances in QEMU
- [x] node discovery / handshake
- [x] bidirectional transport abstraction
- [ ] framing and message protocol
- [ ] heterogeneous architecture support
- [ ] distributed task execution
- [ ] resource discovery
- [ ] distributed ML workloads

![Basic data exchange between nodes](media/msg.gif)

The exact order may change as the kernel evolves.

## Philosophy

Taurine is intentionally small and experimental.

There is no goal of implementing every feature expected from a conventional Unix-like operating system. Subsystems should exist because they enable something the project actually needs.

If a scheduler is not needed yet, there is no reason to build one merely because operating systems normally have schedulers. If a filesystem becomes necessary for persistent state and executable loading, then the filesystem gets built.

The objective is to understand and control the entire stack — from boot and interrupts to communication and eventually distributed computation.

## Building

> [!IMPORTANT]
> The kernel is built with **nightly** Rust and Cargo.

The project contains separate `kernel` and `spawn` components. The `spawn` component produces the bootable image used during development.

Build the image with:

```powershell
cd spawn
cargo build
cd ..
```
Emulate the image according to [explanation above.](#startup)
QEMU is highly recommended for development and testing.
