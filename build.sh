#!/usr/bin/env bash
set -e

OUTPUT_DIRECTORY="$CARGO_MANIFEST_DIR/target/out"
UEFI_FILE="$1"

# Clean up previous builds
rm -rf "$OUTPUT_DIRECTORY"
mkdir -p "$OUTPUT_DIRECTORY"

# Create the bootable image with an partition as EFI System
dd if=/dev/zero of="$OUTPUT_DIRECTORY/bootable.img" bs=1M count=10
fdisk "$OUTPUT_DIRECTORY/bootable.img" <<EOF
g
n
1
2048
18432
t
1
w
EOF

# Format the partition as FAT32 and instert back
dd if="$OUTPUT_DIRECTORY/bootable.img" of="$OUTPUT_DIRECTORY/partition.img" bs=512 skip=2048 count=16384
mkfs.fat "$OUTPUT_DIRECTORY/partition.img"

# Mount the partition file
mkdir -p "$OUTPUT_DIRECTORY/mount/"
sudo mount "$OUTPUT_DIRECTORY/partition.img" "$OUTPUT_DIRECTORY/mount/"

# Copy the file
sudo mkdir -p "$OUTPUT_DIRECTORY/mount/EFI/BOOT"
sudo cp "$UEFI_FILE" "$OUTPUT_DIRECTORY/mount/EFI/BOOT/BOOTX64.EFI"

# Umount the partition file
sudo umount "$OUTPUT_DIRECTORY/mount/"
rm -r "$OUTPUT_DIRECTORY/mount"

# Insert the partition file to the bootable image
dd if="$OUTPUT_DIRECTORY/partition.img" of="$OUTPUT_DIRECTORY/bootable.img" bs=512 seek=2048 conv=notrunc
rm "$OUTPUT_DIRECTORY/partition.img"

echo "Bootable image created at $OUTPUT_DIRECTORY/bootable.img"