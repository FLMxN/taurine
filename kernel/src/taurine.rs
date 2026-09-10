#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod glucose;
mod adrenaline;
mod noradrenaline;
mod dopamine;
mod nicotine;

use crate::dopamine::FrameBufferWriter;

// use core::arch::asm;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};
use bootloader_api::info::{MemoryRegionKind};
use bootloader_api::{
    BootInfo, entry_point,
    config::{BootloaderConfig, Mapping},
};

static BODY: [u8; 3] = [225, 250, 245];
pub static mut LIFETIME: u64 = 0;
static BLINKING: AtomicBool = AtomicBool::new(false);
const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(entry, config = &BOOTLOADER_CONFIG);

fn entry(boot_info: &'static mut BootInfo) -> ! {
	adrenaline::wake();
	let mut conscious = 0i64;
	let framebuffer = boot_info
		.framebuffer
		.take()
		.expect("framebuffer unavailable");
	let info = framebuffer.info();
	let mut screen = FrameBufferWriter::new(framebuffer.into_buffer(), info);
	screen.clear(BODY);
	screen.write_text(32, 32, b"taurine v0.1.0 kernel by mephisto", [46, 247, 130]);
	let usable_memory = memtotal(boot_info);
	let memory_message = if usable_memory > 0 {
		&b"Usable memory found:"[..]
	} else {
		&b"No usable memory found"[..]
	};

	noradrenaline::brace();

	screen.write_text(32, 56, memory_message, [46, 247, 130]);
	screen.write_text(info.width-128, 56, b"lifetime", [46, 247, 130]);

	if usable_memory > 0 {
		let (digits, first_digit) = u64_to_decimal(usable_memory);

		screen.write_text(32, 80, &digits[first_digit..], [46, 247, 130]);
    }

	loop {
		if eyes_shut() {
			blink(&mut screen);
		}
		unsafe {
		if conscious != LIFETIME as i64 {
			conscious = LIFETIME as i64;
			let (digits, first_digit) = u64_to_decimal(conscious as u64);
			screen.fill([info.width-128, info.width-128 + 8 * 12], [80, 80 + 16], BODY);
			screen.write_text(info.width-128, 80, &digits[first_digit..], [46, 247, 130]);
			}
		}
	}
}


fn memtotal(boot_info: &BootInfo) -> u64 {
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

pub fn chill() {
    unsafe {
        LIFETIME += 1;
    }
}

pub fn blinking() {
	BLINKING.store(true, Ordering::Relaxed);
}

fn eyes_shut() -> bool {
	BLINKING.swap(false, Ordering::Relaxed)
}

pub fn blink(screen: &mut FrameBufferWriter) {
	let command = dopamine::snapshot();
	screen.fill([32, 32 + 8 * 48], [120, 120 + 16], BODY);
	screen.write_text(32, 120, command.as_bytes(), [46, 247, 130]);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	// screen.clear([0, 0, 0]);
	// screen.write_text(32, 32, b"kernel panic! halting...", [46, 247, 130]);
	loop {
		unsafe {
			core::arch::asm!("hlt");
		}
	}
}


