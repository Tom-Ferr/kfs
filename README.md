# KFS — Kernel From Scratch

A 32-bit x86 operating system kernel developed from scratch using Rust and Assembly.

The project explores low-level operating system development, including memory management, interrupt handling, process management, system calls and hardware interaction.

**Status:** Work in progress.

## Features

### Kernel Architecture
- 32-bit x86 architecture.
- Multiboot boot protocol.
- Global Descriptor Table (GDT).
- Interrupt Descriptor Table (IDT).
- Task State Segment (TSS).
- Kernel and user-mode execution.

### Memory Management
- Paging and virtual memory.
- Physical memory management.
- Kernel heap allocation.
- Process address-space management.

### Process Management
- Process Control Blocks (PCB).
- Process creation and termination.
- Process scheduling.
- `fork()` and `wait()` system calls.
- Signal handling.
- Inter-process messaging.

### Interrupts and Hardware Interaction
- Programmable Interval Timer (PIT).
- Keyboard interrupt handling.
- QWERTY and AZERTY keyboard layouts.
- Low-level hardware I/O.

### Console
- VGA text-mode output.
- Interactive command interface.
- Multiple screen buffers.
- Configurable text colors and display modes.

### Additional Components
- Initial EXT2 filesystem implementation.
- Virtual File System (VFS) abstractions.
- IDE-related components.

## Requirements

- Linux
- Docker and Docker Compose (recommended workflow)
- Rust nightly (`nightly-2025-12-01`) (local execution)
- Rust source component (`rust-src`)
- GNU Make
- NASM
- GNU Binutils
- GRUB utilities
- xorriso
- QEMU (`qemu-system-i386`)

## Installation

Clone the repository:

```bash
git clone https://github.com/Tom-Ferr/kfs.git
cd kfs
```

## Build and Run

### Docker (Recommended)

From the repository root, build and launch the kernel in QEMU using Docker:

```bash
make run
```

The QEMU display is accessible through a browser using noVNC:

```text
http://localhost:8080
```

### Local Execution

Alternatively, build and launch the kernel directly on the host machine:

**Warning:** The `make local` target automatically installs the required dependencies if they are missing. This process requires `sudo` privileges.

```bash
make local
```

### Clean

Remove intermediate build files:

```bash
make clean
```

Remove generated binaries and build files:

```bash
make fclean
```

## Interactive Commands

Once the kernel has booted, the console provides an interactive command interface.

| Command | Description |
|---|---|
| `dark` / `light` | Change display mode |
| `clear` | Clear the console |
| `stack` | Display stack information |
| `ps` | Display process information |
| `fork` | Test process creation |
| `wait` | Test process synchronization |
| `signal` | Test signal handling |
| `kill` | Test process termination |
| `test` | Run process-related tests |
| `sleep` | Suspend execution |
| `azerty` / `qwerty` | Change keyboard layout |
| `halt` | Halt the CPU |
| `reboot` | Reboot the system |
| `shutdown` | Shut down the system |

Additional commands are available for changing console text colors.