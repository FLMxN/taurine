use core::arch::asm;
use heapless::{String, format};
use bootloader_api::info::MemoryRegionKind;
use crate::BOOTINFO;
use crate::ID_STR;
use crate::LIFETIME;
use crate::{u64_to_bytes, FILESYSTEM};
use crate::electrons::serotonine;
use crate::electrons::dopamine;

pub unsafe fn execute(prompt: &str) -> String<32> {
    if prompt.contains("=") {
    // let mut args: [&str; 2] = ["NONE", "NONE"];
    let (function, argument) = prompt
        .split_once('=')
        .unwrap_or((prompt, ""));
    match function {
        "MSG" => return msg(argument),
        "WRITE" => return write(argument),
        "READ" => return read(argument),
        "POKE" => return poke(argument),
        _ => return String::new(),
        }
    } else {
    match prompt {
        "FAINT" => faint(),
        "MEMTOTAL" => return memtotal(),
        "UPTIME" => return uptime(),
        "PING" => return ping(),
        _ => return String::new(),
        }
    }
}

unsafe fn write(args: &str) -> String<32> {
    let fileguard = FILESYSTEM.lock();
    if let Some((path, content)) = args.split_once(' ') {
    if let Some(fs) = fileguard.as_ref() {
        return match dopamine::internalize(fs, path, content.as_bytes()) {
            Ok(()) => String::try_from("DONE").unwrap(),
            Err(_) => String::try_from("FAILED").unwrap(),
        };
    }
}

    String::try_from("FAILED").unwrap()
}

fn read(args: &str) -> String<32> {
    let mut output = [0u8; 128];
    let fileguard = FILESYSTEM.lock();

    if let Some(fs) = fileguard.as_ref() {
        if let Ok(size) = dopamine::recall(fs, args, &mut output) {
            let content = core::str::from_utf8(&output[..size]).unwrap_or("");
            return String::try_from(content).unwrap_or_else(|_| String::try_from("FAILED").unwrap());
        }
    }

    String::try_from("FAILED").unwrap()
}

fn poke(args: &str) -> String<32> {
    let Some((address, value)) = args.split_once(' ') else {
        return String::try_from("FAILED").unwrap();
    };
    let (Some(address), Some(value)) = (parse_integer(address), parse_integer(value)) else {
        return String::try_from("FAILED").unwrap();
    };
    if address % core::mem::align_of::<u64>() as u64 != 0 {
        return String::try_from("FAILED").unwrap();
    }

    unsafe {
        asm!(
            "mov qword ptr [{address}], {value}",
            address = in(reg) address,
            value = in(reg) value,
            options(nostack),
        );
    }
    String::try_from("DONE").unwrap()
}

fn parse_integer(input: &str) -> Option<u64> {
    let input = input.trim();
    if let Some(hex) = input.strip_prefix("0x").or_else(|| input.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        input.parse().ok()
    }
}

fn msg(content: &str) -> String<32> {
    let message = format!(32; "MSG={}={}\n", ID_STR.lock().as_str(), content).unwrap();
    serotonine::send_bytes(message.as_bytes());
    String::try_from("SENT").unwrap()
}
unsafe fn uptime() -> String<32> {
    let (digits, first_digit) = unsafe {u64_to_bytes(LIFETIME)};
    digits[first_digit..].iter().map(|&d| d as char).collect::<String<32>>()
}

unsafe fn memtotal() -> String<32> {
    let boot_info = unsafe {
        (&raw const BOOTINFO)
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
    };
    let usable_memory: u64 = boot_info
		.memory_regions
		.iter()
		.filter(|region| region.kind == MemoryRegionKind::Usable)
		.map(|region| region.end - region.start)
		.sum();

    let (digits, first_digit) = u64_to_bytes(usable_memory);
    digits[first_digit..].iter().map(|&d| d as char).collect::<String<32>>()
}

fn ping() -> String<32> {
    serotonine::send_bytes(b"PING\n");
    String::try_from("PONG").unwrap()
}

fn faint() -> ! {
    unsafe {
        loop {
            let status: u8;
            asm!(
                "in al, dx",
                in("dx") 0x64u16,
                out("al") status,
            );

            if status & 0x02 == 0 {
                break;
            }
        }

        asm!(
            "mov al, 0xfe",
            "out 0x64, al",
            options(nostack, nomem),
        );
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}