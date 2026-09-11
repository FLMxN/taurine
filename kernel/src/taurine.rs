#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod glucose;
mod adrenaline;
mod noradrenaline;
mod dopamine;
mod nicotine;

use crate::dopamine::FrameBufferWriter;
use crate::dopamine::LAST_COMMAND;
use crate::dopamine::PRINTABLE;

// use core::arch::asm;
use core::panic::PanicInfo;
use heapless::format;
use core::sync::atomic::{AtomicBool, Ordering};
use bootloader_api::{
    BootInfo, entry_point,
    config::{BootloaderConfig, Mapping},
};

const VERSION: &str = const_env::env_lit!("VERSION", "0.0.0");

static BODY: [u8; 3] = [225, 250, 245];
pub static mut LIFETIME: u64 = 0;
static BLINKING: AtomicBool = AtomicBool::new(false);
pub static mut CURSOR_LINE: usize = 1;
pub static mut BOOTINFO: Option<&'static mut BootInfo> = None;
const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(entry, config = &BOOTLOADER_CONFIG);

fn entry(boot_info: &'static mut BootInfo) -> ! {
	adrenaline::wake();
	let mut conscious = 0i64;
	unsafe {
		BOOTINFO = Some(boot_info);
		let framebuffer = (&raw mut BOOTINFO)
			.as_mut()
			.unwrap()
			.as_mut()
			.unwrap()
			.framebuffer
			.take()
			.expect("framebuffer unavailable");
	let info = framebuffer.info();
	let mut screen = FrameBufferWriter::new(framebuffer.into_buffer(), info);
	screen.clear(BODY);
	let version_text = format!(64; "taurine v{} kernel by mephisto", VERSION).unwrap();
	screen.write_text(32, 32, version_text.as_bytes(), [46, 247, 130]);
	
	noradrenaline::brace();

	screen.write_text(info.width-128, 56, b"lifetime", [46, 247, 130]);
	screen.write_text(32, 120 + 16 * CURSOR_LINE, b"% ", [46, 247, 130]);

	loop {
		if eyes_shut() {
			blink(&mut screen);
		}
		if conscious != LIFETIME as i64 {
			conscious = LIFETIME as i64;
			let (digits, first_digit) = u64_to_decimal(conscious as u64);
			screen.fill([info.width-128, info.width-128 + 8 * 12], [80, 80 + 16], BODY);
			screen.write_text(info.width-128, 80, &digits[first_digit..], [46, 247, 130]);
			}
		}
	}
}


// pub fn memtotal(boot_info: &BootInfo) -> u64 {
// 	boot_info
// 		.memory_regions
// 		.iter()
// 		.filter(|region| region.kind == MemoryRegionKind::Usable)
// 		.map(|region| region.end - region.start)
// 		.sum()
// }

pub fn u64_to_decimal(mut value: u64) -> ([u8; 20], usize) {
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
	unsafe {
	// screen.fill([32, (32 + 8 * 48)+16*CURSOR_LINE], [120, (120 + 16)+16*CURSOR_LINE], BODY);
	screen.write_text(32, 120+16*CURSOR_LINE, command.0.as_bytes(), [46, 247, 130]);
		if command.1 {CURSOR_LINE += 1;}
		screen.write_text(32, 120+16*CURSOR_LINE, b"% ", [46, 247, 130]);
		LAST_COMMAND.lock().clear();
		PRINTABLE.lock().clear();
	}
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


