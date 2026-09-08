#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod glucose;
mod adrenaline;
mod noradrenaline;

// use core::arch::asm;
use core::panic::PanicInfo;
use bootloader_api::info::{FrameBufferInfo, MemoryRegionKind, PixelFormat};
use bootloader_api::{
    BootInfo, entry_point,
    config::{BootloaderConfig, Mapping},
};

static BODY: [u8; 3] = [225, 250, 245];
pub static mut LIFETIME: u64 = 0;
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

struct FrameBufferWriter {
	buffer: &'static mut [u8],
	info: FrameBufferInfo,
}

impl FrameBufferWriter {
	fn new(buffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
		Self { buffer, info }
	}

	fn fill(&mut self, xx: [usize; 2], yy: [usize; 2], color: [u8; 3]) {
		for y in yy[0]..yy[1] {
			for x in xx[0]..xx[1] {
				self.set_pixel(x, y, color);
			}
		}
	}

	fn clear(&mut self, color: [u8; 3]) {
		let bg_color = [color[0] + 30, color[1] + 5, color[2] + 10];
		for y in 0..self.info.height {
			for x in 0..self.info.width {
				self.set_pixel(x, y, bg_color);
			}
		}
		for y in 8..self.info.height-8 {
			for x in 8..self.info.width-8 {
				self.set_pixel(x, y, color);
			}
		}
	}

	fn write_text(&mut self, x: usize, y: usize, text: &[u8], color: [u8; 3]) {
		let mut cursor_x = x;
		for &byte in text {
			self.write_char(cursor_x, y, byte, color);
			cursor_x += 12;
		}
	}

	fn write_char(&mut self, x: usize, y: usize, byte: u8, color: [u8; 3]) {
		for (row, bits) in glyph(byte).iter().enumerate() {
			for column in 0..5 {
				if bits & (1 << (4 - column)) != 0 {
					for scale_y in 0..2 {
						for scale_x in 0..2 {
							self.set_pixel(x + column * 2 + scale_x, y + row * 2 + scale_y, color);
						}
					}
				}
			}
		}
	}

	fn set_pixel(&mut self, x: usize, y: usize, color: [u8; 3]) {
		if x >= self.info.width || y >= self.info.height {
			return;
		}

		let offset = (y * self.info.stride + x) * self.info.bytes_per_pixel;
		let pixel = &mut self.buffer[offset..offset + self.info.bytes_per_pixel];
		match self.info.pixel_format {
			PixelFormat::Rgb => pixel[..3].copy_from_slice(&color),
			PixelFormat::Bgr => pixel[..3].copy_from_slice(&[color[2], color[1], color[0]]),
			PixelFormat::U8 => pixel[0] = color[0],
			PixelFormat::Unknown { red_position, green_position, blue_position } => {
				let value = (u32::from(color[0]) << red_position)
					| (u32::from(color[1]) << green_position)
					| (u32::from(color[2]) << blue_position);
				for (index, byte) in value.to_le_bytes().iter().enumerate() {
					if index < pixel.len() {
						pixel[index] = *byte;
					}
				}
			}
			_ => pixel[0] = color[0],
		}
	}
}

fn glyph(byte: u8) -> [u8; 7] {
	match byte.to_ascii_uppercase() {
		b'A' => [14, 17, 17, 31, 17, 17, 17], b'B' => [30, 17, 17, 30, 17, 17, 30],
		b'C' => [14, 17, 16, 16, 16, 17, 14], b'D' => [30, 17, 17, 17, 17, 17, 30],
		b'E' => [31, 16, 16, 30, 16, 16, 31], b'F' => [31, 16, 16, 30, 16, 16, 16],
		b'G' => [14, 17, 16, 23, 17, 17, 14], b'H' => [17, 17, 17, 31, 17, 17, 17],
		b'I' => [31, 4, 4, 4, 4, 4, 31], b'J' => [7, 2, 2, 2, 2, 18, 12],
		b'K' => [17, 18, 20, 24, 20, 18, 17], b'L' => [16, 16, 16, 16, 16, 16, 31],
		b'M' => [17, 27, 21, 21, 17, 17, 17], b'N' => [17, 25, 21, 19, 17, 17, 17],
		b'O' => [14, 17, 17, 17, 17, 17, 14], b'P' => [30, 17, 17, 30, 16, 16, 16],
		b'Q' => [14, 17, 17, 17, 21, 18, 13], b'R' => [30, 17, 17, 30, 20, 18, 17],
		b'S' => [15, 16, 16, 14, 1, 1, 30], b'T' => [31, 4, 4, 4, 4, 4, 4],
		b'U' => [17, 17, 17, 17, 17, 17, 14], b'V' => [17, 17, 17, 17, 17, 10, 4],
		b'W' => [17, 17, 17, 21, 21, 21, 10], b'X' => [17, 17, 10, 4, 10, 17, 17],
		b'Y' => [17, 17, 10, 4, 4, 4, 4], b'Z' => [31, 1, 2, 4, 8, 16, 31],
		b'0' => [14, 17, 19, 21, 25, 17, 14], b'1' => [4, 12, 4, 4, 4, 4, 14],
		b'2' => [14, 17, 1, 2, 4, 8, 31], b'3' => [30, 1, 1, 14, 1, 1, 30],
		b'4' => [2, 6, 10, 18, 31, 2, 2], b'5' => [31, 16, 16, 30, 1, 1, 30],
		b'6' => [14, 16, 16, 30, 17, 17, 14], b'7' => [31, 1, 2, 4, 8, 8, 8],
		b'8' => [14, 17, 17, 14, 17, 17, 14], b'9' => [14, 17, 17, 15, 1, 1, 14],
		b'.' => [0, 0, 0, 0, 0, 6, 6], b':' => [0, 6, 6, 0, 6, 6, 0],
		_ => [0, 0, 0, 0, 0, 0, 0],
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


