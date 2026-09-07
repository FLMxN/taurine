#![no_std]
#![no_main]

mod glucose;

use core::panic::PanicInfo;
use bootloader_api::info::MemoryRegionKind;
use bootloader_api::{
    BootInfo, entry_point,
    config::{BootloaderConfig, Mapping},
};

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(entry, config = &BOOTLOADER_CONFIG);

fn entry(boot_info: &'static mut BootInfo) -> ! {
	let physical_memory_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("physical memory mapping unavailable");

    let vga_buffer = (physical_memory_offset + 0xb8000) as *mut u8;
	let message = b"taurine v0.1.0 kernel by mephisto";
	let usable_memory = mem_total(boot_info);
	let memory_message = if usable_memory > 0 {
		&b"Usable memory found:"[..]
	} else {
		&b"No usable memory found"[..]
	};

	for (index, &byte) in message.iter().enumerate() {
		let offset = index * 2;

		unsafe {
			core::ptr::write_volatile(vga_buffer.add(offset), byte);
			core::ptr::write_volatile(vga_buffer.add(offset + 1), 0x0f);
		}
	}

	for (index, &byte) in memory_message.iter().enumerate() {
		let offset = 80 * 2 + index * 2;

		unsafe {
			core::ptr::write_volatile(vga_buffer.add(offset), byte);
			core::ptr::write_volatile(vga_buffer.add(offset + 1), 0x0f);
		}
	}

	if usable_memory > 0 {
		let (digits, first_digit) = u64_to_decimal(usable_memory);

		for (index, &byte) in digits[first_digit..].iter().enumerate() {
			let offset = 80 * 4 + index * 2;

			unsafe {
				core::ptr::write_volatile(vga_buffer.add(offset), byte);
				core::ptr::write_volatile(vga_buffer.add(offset + 1), 0x0f);
			}
		}
    }

	loop {
		core::hint::spin_loop();
	}
}

fn mem_total(boot_info: &BootInfo) -> u64 {
	boot_info
		.memory_regions
		.iter()
		.filter(|region| region.kind == MemoryRegionKind::Usable)
		.map(|region| region.end - region.start)
		.sum()
}

fn u64_to_decimal(mut value: u64) -> ([u8; 20], usize) {
	let mut digits = [0u8; 20];
	let mut first_digit = digits.len();

	if value == 0 {
		digits[0] = b'0';
		return (digits, 0);
	}

	while value > 0 {
		first_digit -= 1;
		digits[first_digit] = b'0' + (value % 10) as u8;
		value /= 10;
	}

	(digits, first_digit)
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	loop {
		core::hint::spin_loop();
	}
}


