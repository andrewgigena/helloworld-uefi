#![no_std]
#![no_main]
#![forbid(unsafe_code)]
extern crate alloc;
extern crate core;

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Write;
use core::str::SplitWhitespace;
use hashbrown::HashMap;
use uefi::prelude::*;
use uefi::proto::console::text::{Color, Key, Output};
use uefi::table::boot::MemoryType;
use uefi::table::runtime::{Time};

const ENTER_KEY: u16 = 0xD;
const RETURN_KEY: u16 = 0x08;

#[entry]
fn efi_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    match uefi::helpers::init(&mut system_table) {
        Ok(_) => (),
        Err(e) => {
            system_table.stdout().write_str("Error initializing UEFI helpers: ").unwrap();
            system_table.stdout().write_str(&format!("{}", e)).unwrap();
            return Status::ABORTED;
        }
    }

    let mut variables = HashMap::new();
    let stdout = system_table.stdout();
    stdout.clear().unwrap();
    help(&mut system_table);


    loop {
        let stdout = system_table.stdout();
        stdout.write_str("\n> ").unwrap();

        let command = get_command(&mut system_table);
        let mut command_parts = command.split_whitespace();
        match command_parts.next() {
            Some("hello_world") => hello_message(&mut system_table),
            Some("system_info") => system_info(&mut system_table),
            Some("memory_info") => memory_info(&mut system_table),
            Some("read_var") => read_var(&mut system_table, &mut variables, command_parts),
            Some("set_var") => set_var(&mut system_table, &mut variables, command_parts),
            Some("delete_var") => delete_var(&mut system_table, &mut variables, command_parts),
            Some("calc") => calculator(&mut system_table, command_parts),
            Some("help") => help(&mut system_table),
            Some("clear") => clear(&mut system_table),
            Some("exit") => break,
            _ => command_not_found(&mut system_table)
        }
    }

    Status::SUCCESS
}

fn clear(system_table: &mut SystemTable<Boot>){
    let stdout = system_table.stdout();
    stdout.clear().unwrap();
}

fn command_not_found(system_table: &mut SystemTable<Boot>){
    let stdout = system_table.stdout();
    stdout.write_str("Command not found.").unwrap()
}

fn help(system_table: &mut SystemTable<Boot>){
    let stdout = system_table.stdout();
    stdout.set_color(Color::White, Color::Black).unwrap();
    stdout.write_str("Available commands:\n").unwrap();
    stdout.write_str("\t- hello_world: Print a hello message\n").unwrap();
    stdout.write_str("\t- system_info: Shows system information\n").unwrap();
    stdout.write_str("\t- memory_info: Shows memory information\n").unwrap();
    stdout.write_str("\t- read_var <name>: Read an variable\n").unwrap();
    stdout.write_str("\t- set_var <name> <value>: Set an variable\n").unwrap();
    stdout.write_str("\t- delete_var <name>: Delete an variable\n").unwrap();
    stdout.write_str("\t- calc <expression>: Calculates some math\n").unwrap();
    stdout.write_str("\t- help: Prints this help\n").unwrap();
    stdout.write_str("\t- clear: Clear the screen\n").unwrap();
    stdout.write_str("\t- exit: Finish the program\n").unwrap();
}

fn system_info(system_table: &mut SystemTable<Boot>) {
    let mut info: Vec<String> = vec![];
    info.push(format!("UEFI Revision: {}", system_table.uefi_revision()));
    info.push(format!(
        "UEFI Firmware Vendor: {}",
        system_table.firmware_vendor()
    ));
    info.push(format!(
        "UEFI Firmware Revision: {:x}",
        system_table.firmware_revision()
    ));
    let time: uefi::Result<Time> = system_table.runtime_services().get_time();
    match time {
        Ok(time) => info.push(format!(
            "Current Time: {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            time.year(),
            time.month(),
            time.day(),
            time.hour(),
            time.minute(),
            time.second()
        )),
        Err(e) => {
            system_table.stdout().write_str("Error getting time: ").unwrap();
            system_table.stdout().write_str(&format!("{}", e)).unwrap();
        }
    }

    let stdout = system_table.stdout();
    stdout.set_color(Color::White, Color::Green).unwrap();
    for line in info {
        stdout.write_str(&line).unwrap();
        stdout.write_str("\n").unwrap();
    }
    stdout.set_color(Color::White, Color::Black).unwrap();
}

fn hello_message(system_table: &mut SystemTable<Boot>) {
    let stdout: &mut Output = system_table.stdout();
    stdout.set_color(Color::White, Color::Blue).unwrap();
    stdout.write_str("¡Hola Mundo!\n").unwrap();
    stdout.set_color(Color::White, Color::Black).unwrap();
}

fn memory_info(system_table: &mut SystemTable<Boot>) {
    let mem_map = system_table.boot_services().memory_map(MemoryType::LOADER_DATA);
    match mem_map {
        Ok(map) => {
            for descriptor in map.entries() {
                let info = format!(
                    "Type: {:?}, Physical Start: {:x}, Virtual Start: {:x}, Page Count: {}, Attribute: {:x}\n",
                    descriptor.ty,
                    descriptor.phys_start,
                    descriptor.virt_start,
                    descriptor.page_count,
                    descriptor.att
                );
                let stdout = system_table.stdout();
                stdout.write_str(&info).unwrap();

                let boot_services = system_table.boot_services();
                boot_services.stall(200_000);
            }
        }
        Err(e) => {
            system_table
                .stdout()
                .write_str(&format!("Error getting memory map: {}\n", e))
                .unwrap();
        }
    }
}
fn read_var(system_table: &mut SystemTable<Boot>, variables: &mut HashMap<String, String>, mut command_parts: SplitWhitespace) {
    if let Some(name) = command_parts.next() {
        let stdout = system_table.stdout();
        match variables.get(name) {
            Some(value) => {
                stdout.write_str(&format!("{} = {}\n", name, value)).unwrap();
            },
            None => {
                stdout.write_str("Variable not found\n").unwrap();
            }
        }
    } else {
        let stdout = system_table.stdout();
        stdout.write_str("Usage: read_var <name>\n").unwrap();
    }
}

fn set_var(system_table: &mut SystemTable<Boot>, variables: &mut HashMap<String, String>, mut command_parts: SplitWhitespace) {
    if let (Some(name), Some(value)) = (command_parts.next(), command_parts.next()) {
        variables.insert(name.parse().unwrap(), value.parse().unwrap());
        let stdout = system_table.stdout();
        stdout.write_str("Variable set\n").unwrap();
    } else {
        let stdout = system_table.stdout();
        stdout.write_str("Usage: set_var <name> <value>\n").unwrap();
    }
}

fn delete_var(system_table: &mut SystemTable<Boot>, variables: &mut HashMap<String, String>, mut command_parts: SplitWhitespace) {
    if let Some(name) = command_parts.next() {
        if variables.remove(name).is_some() {
            let stdout = system_table.stdout();
            stdout.write_str("Variable deleted\n").unwrap();
        } else {
            let stdout = system_table.stdout();
            stdout.write_str("Variable not found\n").unwrap();
        }
    } else {
        let stdout = system_table.stdout();
        stdout.write_str("Usage: delete_var <name>\n").unwrap();
    }
}

fn calculator(system_table: &mut SystemTable<Boot>, command_parts: SplitWhitespace) {
    let result = evaluate_expression(command_parts);
    let stdout = system_table.stdout();
    match result {
        Ok(value) => {
            stdout.write_str(&format!("Result: {}\n", value)).unwrap();
        },
        Err(e) => {
            stdout.write_str(&format!("Error evaluating expression: {}\n", e)).unwrap();
        }
    }
}

fn evaluate_expression(expression: SplitWhitespace) -> Result<i32, &str> {
    let tokens: Vec<&str> = expression.collect();
    if tokens.len() != 3 {
        return Err("Invalid expression format. Use: <number> <operator> <number>");
    }

    let a = tokens[0].parse::<i32>().map_err(|_| "Invalid number")?;
    let op = tokens[1];
    let b = tokens[2].parse::<i32>().map_err(|_| "Invalid number")?;

    match op {
        "+" => Ok(a + b),
        "-" => Ok(a - b),
        "*" => Ok(a * b),
        "/" => if b != 0 { Ok(a / b) } else { Err("Division by zero") },
        _ => Err("Invalid operator"),
    }
}

fn get_command(system_table: &mut SystemTable<Boot>) -> String {
    let mut command = String::new();
    loop {
        match system_table.stdin().read_key() {
            Ok(Some(Key::Printable(char))) => {
                let value: u16 = char.into();
                match value {
                    ENTER_KEY => {
                        system_table.stdout().write_char('\n').unwrap();
                        break;
                    }
                    RETURN_KEY => {
                        command.pop();
                        system_table.stdout().write_str("\x08 \x08").unwrap();
                    }
                    _ => {
                        let char: char = char.into();
                        command.push(char);
                        system_table.stdout().write_char(char).unwrap();
                    }
                }
            }
            Err(e) => {
                system_table.stdout().write_str("Error reading key: ").unwrap();
                system_table.stdout().write_str(&format!("{}", e)).unwrap();
            }
            _ => {}
        }
    }
    command
}

