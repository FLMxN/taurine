# Taurine

## Master / Slave Startup

Build the image first (via PS):

```powershell
cd spawn
cargo build
cd ..
```

Start the **master** first. It creates the TCP endpoint:

```powershell
qemu-system-x86_64.exe `
  -drive format=raw,file=.\taurine.img `
  -serial tcp:127.0.0.1:4555,server=on,wait=off
```

Start the **slave** second. It connects to the master's endpoint:

```powershell
qemu-system-x86_64.exe `
  -drive format=raw,file=.\taurine.img `
  -serial tcp:127.0.0.1:4555
```

The master and slave use the same kernel image. Their role is determined by the QEMU serial connection mode, not by a kernel flag.
