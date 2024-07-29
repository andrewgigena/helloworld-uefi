use clap::Parser;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;
use gpt::disk::LogicalBlockSize;
use gpt::{GptConfig, partition_types};
use fatfs::{FileSystem, format_volume, FormatVolumeOptions, FsOptions};
use fscommon::BufStream;

const IMG_BLOCK_SIZE: u64 = 512;
const IMG_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const EFI_PART_SIZE: u64 = 8 * 1024 * 1024; // 8MB
const EFI_PART_ALIGNMENT: u64 = 2048;
const EFI_PART_START: u64 = 2048 * IMG_BLOCK_SIZE;
const EFI_PART_END: u64 = EFI_PART_START + EFI_PART_SIZE;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
struct Args {
    /// Path to the .efi file
    efi_file: String,

    /// Path to the output directory
    #[clap(long, default_value = "target/out")]
    output_directory: String,

    /// Filename of the output image
    #[clap(long, default_value = "bootable.img")]
    output_filename: String,

    /// Run mode: 'build' or 'run'
    #[clap(long, default_value = "build")]
    mode: String,

    /// Path to OVMF.fd file
    #[clap(long, default_value = "OVMF.fd")]
    ovmf_path: PathBuf,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    match args.mode.as_str() {
        "build" => build_image(&args),
        "run" => run_qemu(&args),
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid mode")),
    }
}

fn build_image(args: &Args) -> io::Result<()> {
    let efi_file = &args.efi_file;
    let output_directory = Path::new(&args.output_directory);
    let output_filename = output_directory.join(&args.output_filename);

    // Clean up previous builds
    fs::remove_dir_all(output_directory).ok();
    fs::create_dir_all(output_directory)?;

    // Setup in-memory space
    let mut mem_device = Box::new(std::io::Cursor::new(vec![0u8; IMG_SIZE as usize]));

    // Create a protective MBR at LBA0
    let mbr = gpt::mbr::ProtectiveMBR::with_lb_size(u32::try_from(20480 - 1).unwrap_or(0xFF_FF_FF_FF));
    mbr.overwrite_lba0(&mut mem_device).expect("failed to write MBR");

    let mut cfg = GptConfig::default()
        .writable(true)
        .logical_block_size(LogicalBlockSize::Lb512)
        .create_from_device(mem_device, None)
        .expect("failed to create GptDisk");

    // Initialize the headers using a blank partition table
    cfg.update_partitions(std::collections::BTreeMap::<u32, gpt::partition::Partition>::new())
        .expect("failed to initialize blank partition table");

    // Add an EFI Partition
    cfg.add_partition("uefi_partition",
                      EFI_PART_SIZE,
                      partition_types::EFI,
                      0,
                      Option::from(EFI_PART_ALIGNMENT))
        .expect("failed to add UEFI partition");

    // Write the partition table and take ownership of the underlying file
    let mut mem_device = cfg.write().expect("failed to write partition table");
    mem_device.seek(SeekFrom::Start(0)).expect("failed to seek");
    let mut final_bytes = vec![0u8; IMG_SIZE as usize];
    mem_device.read_exact(&mut final_bytes).expect("failed to read contents of memory device");

    // Write the final bytes to a file
    let mut file = File::create(output_filename.clone())?;
    file.write_all(&final_bytes)?;

    // Reopen the file for further modifications
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(output_filename.clone())?;

    // Locate the EFI partition and format it as FAT32
    let mut buf_stream = BufStream::new(fscommon::StreamSlice::new(file, EFI_PART_START, EFI_PART_END)?);

    format_volume(&mut buf_stream, FormatVolumeOptions::new().bytes_per_cluster(512))?;

    // Reopen the file for further modifications
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(output_filename.clone())?;

    // Add the .efi file to the EFI partition
    let buf_stream = BufStream::new(fscommon::StreamSlice::new(file, EFI_PART_START, EFI_PART_END)?);
    let fs = FileSystem::new(buf_stream, FsOptions::new())?;

    let root_dir = fs.root_dir();
    let efi_dir = root_dir.create_dir("EFI")?;
    let boot_dir = efi_dir.create_dir("BOOT")?;
    let mut boot_file = boot_dir.create_file("BOOTX64.EFI")?;

    let mut src_file = File::open(efi_file)?;
    std::io::copy(&mut src_file, &mut boot_file)?;

    println!("Bootable image created at {}", output_filename.display());
    Ok(())
}

fn run_qemu(args: &Args) -> io::Result<()> {
    let output_directory = Path::new(&args.output_directory);
    let output_filename = output_directory.join(&args.output_filename);

    let mut qemu_command = Command::new("qemu-system-x86_64");
    qemu_command
        .arg("-enable-kvm")
        .arg("-bios")
        .arg(&args.ovmf_path)
        .arg("-drive")
        .arg(format!("format=raw,file={}", output_filename.display()));

    println!("Running QEMU with command: {:?}", qemu_command);
    qemu_command.status()?;

    Ok(())
}