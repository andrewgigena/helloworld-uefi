#!/usr/bin/env bash
set -e
UEFI_FILE="$1"
OUTPUT_DIRECTORY="$CARGO_MANIFEST_DIR/target/out"

# Determine if running in release mode
if [ "$CARGO_PROFILE" == "release" ]; then
    echo "Building the EFI image"
    ./build.sh "$UEFI_FILE"
    echo "Running the EFI image"
    qemu-system-x86_64 -enable-kvm -bios OVMF.fd -drive format=raw,file="$OUTPUT_DIRECTORY/bootable.img"
else
    echo "Running the EFI directly"
    mkdir -p "$OUTPUT_DIRECTORY/esp/efi/boot"
    cp "$1" "$OUTPUT_DIRECTORY/esp/efi/boot/bootx64.efi"
    qemu-system-x86_64 -enable-kvm -bios OVMF.fd -drive format=raw,file=fat:rw:"$OUTPUT_DIRECTORY/esp"
fi