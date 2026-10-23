#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod electrons;
mod nucleus;

use crate::electrons::dopamine::FATBuffer;
use crate::electrons::glutamate::FrameBufferWriter;
use crate::electrons::glutamate::PRINTABLE;

// use core::arch::asm;
use core::fmt::Write;
use core::panic::PanicInfo;
use spin::Mutex;
use heapless::{format, String};
use core::sync::atomic::{AtomicBool, Ordering};
use bootloader_api::{
    BootInfo, entry_point,
    config::{BootloaderConfig, Mapping},
};
use core3::io::{Cursor};
use fatfs::{FileSystem, FormatVolumeOptions, FsOptions, ReadWriteSeek};

type FAT12 = FileSystem<FATBuffer<'static>>;

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};
const VERSION: &str = const_env::env_lit!("VERSION", "0.0.0");

static BODY: [u8; 3] = [225, 250, 245];
static BLINKING: AtomicBool = AtomicBool::new(false);
static PANICKING: AtomicBool = AtomicBool::new(false);

static mut RAM_VOLUME: [u8; 512 * 1024] = [0; 512 * 1024];
pub static mut LIFETIME: u64 = 0;
pub static mut CURSOR_LINE: usize = 1;
pub static mut BOOTINFO: Option<&'static mut BootInfo> = None;
pub static FILESYSTEM: Mutex<Option<FAT12>> = Mutex::new(None);
pub static ID_STR: Mutex<String<64>> = Mutex::new(String::new());

struct PanicSerial;

impl Write for PanicSerial {
	fn write_str(&mut self, text: &str) -> core::fmt::Result {
		for byte in text.bytes() {
			while panic_in(0x3FD) & 0x20 == 0 {}
			panic_out(0x3F8, byte);
		}
		Ok(())
	}
}

fn panic_out(port: u16, value: u8) {
	unsafe {
		core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
	}
}

fn panic_in(port: u16) -> u8 {
	let value: u8;
	unsafe {
		core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack));
	}
	value
}

entry_point!(entry, config = &BOOTLOADER_CONFIG);

fn entry(boot_info: &'static mut BootInfo) -> ! {
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
	let rsdp = (&raw mut BOOTINFO)
			.as_mut()
			.unwrap()
			.as_mut()
			.unwrap()
			.rsdp_addr
			.take()
			.expect("rsdp unavailable");
	
	let volume = &mut *(&raw mut RAM_VOLUME);
	let buffer = FATBuffer(Cursor::new(&mut volume[..]));
	let fs = acknowledge(buffer).unwrap();
	*FILESYSTEM.lock() = Some(fs);
	{	let fileguard = FILESYSTEM.lock();
		let root = fileguard.as_ref().unwrap().root_dir();
		let _ = root.create_dir("LOGS");
		let _ = electrons::dopamine::internalize(fileguard.as_ref().unwrap(), "LOGS/BOOTLOG.LOG",
		&u64_to_bytes(read_tsc()).0[u64_to_bytes(read_tsc()).1..]);
	}

	let seed = rsdp.wrapping_add(read_tsc());
	let mut id = Id::new(seed);
	let mut id_str: String<64> = String::new();
	for ch in id.next().iter() {	
			id_str.push(*ch as char).unwrap();	
	}
	*ID_STR.lock() = id_str.clone();
	electrons::serotonine::welcome(ID_STR.lock().as_str());

	let info = framebuffer.info();
	let mut screen = FrameBufferWriter::new(framebuffer.into_buffer(), info);
	screen.clear(BODY);
	let version_text = format!(64; "taurine v{} kernel by mephisto", VERSION).unwrap();
	screen.write_text(32, 32, version_text.as_bytes(), [46, 247, 130]);
	let id_text = format!(64; "machine id = {}", ID_STR.lock().as_str()).unwrap();
	screen.write_text(32, 48, id_text.as_bytes(), [46, 247, 130]);

	nucleus::noradrenaline::brace();

	screen.write_text(32, 120 + 16 * CURSOR_LINE, b"% ", [46, 247, 130]);

	loop {
		electrons::serotonine::flush();
		electrons::serotonine::poll();
		while let Some(byte) = electrons::serotonine::check() {
			electrons::serotonine::receive(byte);
		}
		if eyes_shut() {
			blink(&mut screen);
			}
		}
	}
}

pub fn u64_to_bytes(mut value: u64) -> ([u8; 20], usize) {
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

pub fn read_tsc() -> u64 {
	let low: u32;
	let high: u32;
	unsafe {
		core::arch::asm!(
			"rdtsc",
			out("eax") low,
			out("edx") high,
		);
	}
	((high as u64) << 32) | (low as u64)
}

pub struct Id {
    state: u64,
}

impl Id {
    pub const fn new(seed: u64) -> Self {
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
	let command = electrons::glutamate::snapshot();

	unsafe {
	screen.write_text(32, 120+16*CURSOR_LINE, command.0.as_bytes(), [46, 247, 130]);
	if command.1 {CURSOR_LINE += 1;}
	screen.write_text(32, 120+16*CURSOR_LINE, b"% ", [46, 247, 130]);
	PRINTABLE.lock().clear();
	}
}

fn acknowledge<IO>(mut device: IO) -> Result<FileSystem<IO>, fatfs::Error<IO::Error>>
where
    IO: ReadWriteSeek,
{
    let format_options = FormatVolumeOptions::new()
        .total_sectors(1024)
        .volume_label(*b"TAURINE    ");   
    fatfs::format_volume(&mut device, format_options)?;
    
    let fs = FileSystem::new(device, FsOptions::new())?;
    Ok(fs)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	unsafe {
		core::arch::asm!("cli", options(nomem, nostack));
	}

	if !PANICKING.swap(true, Ordering::Relaxed) {
		panic_out(0x3F9, 0x00);
		panic_out(0x3FB, 0x80);
		panic_out(0x3F8, 0x01);
		panic_out(0x3F9, 0x00);
		panic_out(0x3FB, 0x03);
		panic_out(0x3FA, 0xC7);
		panic_out(0x3FC, 0x0B);

		let _ = writeln!(PanicSerial, "\r\nKERNEL PANIC: {info}");
	}

	loop {
		unsafe {
			core::arch::asm!("hlt", options(nomem, nostack));
		}
	}
}

