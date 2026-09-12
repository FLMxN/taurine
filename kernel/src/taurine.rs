#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod glucose;
mod noradrenaline;
mod glutamate;
mod nicotine;
mod serotonine;

use crate::glutamate::FrameBufferWriter;
use crate::glutamate::PRINTABLE;

// use core::arch::asm;
use core::panic::PanicInfo;
use heapless::{format, String};
use core::sync::atomic::{AtomicBool, Ordering};
use bootloader_api::{
    BootInfo, entry_point,
    config::{BootloaderConfig, Mapping},
};

const VERSION: &str = const_env::env_lit!("VERSION", "0.0.0");
// const MODE: &str = const_env::env_lit!("MODE", "unknown");

static BODY: [u8; 3] = [225, 250, 245];
static BLINKING: AtomicBool = AtomicBool::new(false);
pub static mut LIFETIME: u64 = 0;
pub static mut CURSOR_LINE: usize = 1;
pub static mut BOOTINFO: Option<&'static mut BootInfo> = None;
const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(entry, config = &BOOTLOADER_CONFIG);

fn entry(boot_info: &'static mut BootInfo) -> ! {
	let mut id = Id::new(0x12345678);
	let mut id_str: String<64> = String::new();
	for ch in id.next().iter() {	
			id_str.push(*ch as char).unwrap();	
	}
	serotonine::welcome(id_str.as_str());
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
	let id_text = format!(64; "machine id = {}", id_str).unwrap();
	screen.write_text(32, 48, id_text.as_bytes(), [46, 247, 130]);

	noradrenaline::brace();

	screen.write_text(32, 120 + 16 * CURSOR_LINE, b"% ", [46, 247, 130]);

	loop {
		serotonine::flush();
		serotonine::poll();
		while let Some(byte) = serotonine::check() {
			serotonine::receive(byte);
		}
		if eyes_shut() {
			blink(&mut screen);
			}
		}
	}
}

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

pub struct Id {
    state: u32,
}

impl Id {
    pub const fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> [u8; 3] {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;

        [
            b'A' + (x % 26) as u8,
            b'0' + ((x / 26) % 10) as u8,
            b'0' + ((x / 260) % 10) as u8,
        ]
    }
}

pub fn blinking() {
	BLINKING.store(true, Ordering::Relaxed);
}

fn eyes_shut() -> bool {
	BLINKING.swap(false, Ordering::Relaxed)
}

pub fn blink(screen: &mut FrameBufferWriter) {
	let command = glutamate::snapshot();
	unsafe {
	screen.write_text(32, 120+16*CURSOR_LINE, command.0.as_bytes(), [46, 247, 130]);
	if command.1 {CURSOR_LINE += 1;}
	screen.write_text(32, 120+16*CURSOR_LINE, b"% ", [46, 247, 130]);
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


