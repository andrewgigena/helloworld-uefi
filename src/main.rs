#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use core::fmt::Write;
use uefi::prelude::*;
use uefi::proto::console::text::{Color, Output};
use uefi::table::runtime::Time;

#[entry]
fn efi_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).unwrap();
    main(&mut system_table);
    Status::SUCCESS
}

fn print_system_info(system_table: &mut SystemTable<Boot>) {
    let mut info: Vec<String> = vec![];
    info.push(format!("UEFI Revision: {}", system_table.uefi_revision()));
    info.push(format!("UEFI Firmware Vendor: {}", system_table.firmware_vendor()));
    info.push(format!("UEFI Firmware Revision: {:x}", system_table.firmware_revision()));
    let time: uefi::Result<Time> = system_table.runtime_services().get_time();
    match time {
        Ok(time) => info.push(format!("Current Time: {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                                      time.year(), time.month(), time.day(), time.hour(), time.minute(), time.second())),
        Err(_e) => info.push("Current Time: Unavailable".parse().unwrap()),
    }

    let stdout = system_table.stdout();
    stdout.clear().unwrap();
    stdout.set_color(Color::White, Color::Green).unwrap();
    for line in info {
        stdout.write_str(&line).unwrap();
        stdout.write_str("\n").unwrap();
    }
}

fn print_hello_message(system_table: &mut SystemTable<Boot>) {
    let stdout: &mut Output = system_table.stdout();
    stdout.set_color(Color::White, Color::Blue).unwrap();
    stdout.write_str("¡Hola Mundo!\n").unwrap();
}

fn main(system_table: &mut SystemTable<Boot>) {
    print_system_info(system_table);
    print_hello_message(system_table);

    loop {
        system_table.boot_services().stall(1_000_000);
    }
}
