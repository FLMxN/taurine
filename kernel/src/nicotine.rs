use core::arch::asm;
use heapless::String;
use bootloader_api::info::MemoryRegionKind;
use crate::BOOTINFO;
use crate::LIFETIME;
use crate::u64_to_decimal;

pub fn execute(command: &str) -> String<32> {
    unsafe {
    match command {
        "FAINT" => faint(),
        "MEMTOTAL" => memtotal(),
        "UPTIME" => uptime(),
        _ => String::new(),
        }
    }
}

unsafe fn uptime() -> String<32> {
    let (digits, first_digit) = u64_to_decimal(LIFETIME);
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

    let (digits, first_digit) = u64_to_decimal(usable_memory);
    digits[first_digit..].iter().map(|&d| d as char).collect::<String<32>>()
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