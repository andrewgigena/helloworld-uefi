#!/usr/bin/env bash

# Copy the binary
mkdir -p esp/efi/boot
cp "$1" esp/efi/boot/bootx64.efi

# Run QEMU
qemu-system-x86_64 -enable-kvm -bios OVMF.fd -drive format=raw,file=fat:rw:esp

# Clean up
rm -rf "esp/"
