# helloworld-uefi

![Hello World](demo.png)

A basic UEFI application written in Rust for learning [OS-dev](https://wiki.osdev.org/) purposes.

## Requirements

- Rust (with the `x86_64-unknown-uefi` target)
- QEMU

## Building

To build the UEFI image, run:

```bash
cargo efi-build
```

This will create a bootable image in the `target/out` directory.

## Running

To run the UEFI image locally, run:

```bash
cargo efi-preview	
```

This will start a QEMU instance with the UEFI bootable image.