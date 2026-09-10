use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use heapless::String;
use spin::Mutex;
use x86_64::instructions::interrupts;

use crate::blinking;
use crate::nicotine;

static COMMAND: Mutex<String<32>> = Mutex::new(String::new());
static HELD_KEYS: Mutex<[bool; 128]> = Mutex::new([false; 128]);

pub fn snapshot() -> String<32> {
	interrupts::without_interrupts(|| {
		let command = COMMAND.lock();
		let mut snapshot = String::new();
		snapshot.push_str("% ").ok();
		snapshot.push_str(command.as_str()).ok();
		snapshot.push_str("_").ok();
		snapshot
	})
}

pub struct FrameBufferWriter {
	buffer: &'static mut [u8],
	info: FrameBufferInfo,
}

impl FrameBufferWriter {
	pub fn new(buffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
		Self { buffer, info }
	}

	pub fn fill(&mut self, xx: [usize; 2], yy: [usize; 2], color: [u8; 3]) {
		for y in yy[0]..yy[1] {
			for x in xx[0]..xx[1] {
				self.set_pixel(x, y, color);
			}
		}
	}

	pub fn clear(&mut self, color: [u8; 3]) {
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

	pub fn write_text(&mut self, x: usize, y: usize, text: &[u8], color: [u8; 3]) {
		let mut cursor_x = x;
		for &byte in text {
			self.write_char(cursor_x, y, byte, color);
			cursor_x += 12;
		}
	}

	fn write_char(&mut self, x: usize, y: usize, byte: u8, color: [u8; 3]) {
		for (row, bits) in articulate(byte).iter().enumerate() {
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

fn articulate(byte: u8) -> [u8; 7] {
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
		b'%' => [24, 25, 2, 4, 8, 19, 3], b'_' => [0, 0, 0, 0, 0, 0, 31],
		_ => [0, 0, 0, 0, 0, 0, 0],
	}
}

pub fn touch(code: u8) {
	let scancode = code & 0x7F;
	let mut held_keys = HELD_KEYS.lock();
	if code & 0x80 != 0 {
		held_keys[scancode as usize] = false;
		return;
	}
	if held_keys[scancode as usize] {
		return;
	}
	held_keys[scancode as usize] = true;
	drop(held_keys);
	if scancode == 0x1C {
		let mut command = COMMAND.lock();
		if command.as_bytes() == b"FAINT" {
			nicotine::faint();
		}
		command.clear();
		blinking();
		return;
	}

	let key = match scancode {
		0x02 => Some('1'),
		0x03 => Some('2'),
		0x04 => Some('3'),
		0x05 => Some('4'),
		0x06 => Some('5'),
		0x07 => Some('6'),
		0x08 => Some('7'),
		0x09 => Some('8'),
		0x0A => Some('9'),
		0x0B => Some('0'),
		0x0C => Some('-'),
		0x0D => Some('='),
        
		0x10 => Some('Q'),
		0x11 => Some('W'),
		0x12 => Some('E'),
		0x13 => Some('R'),
		0x14 => Some('T'),
		0x15 => Some('Y'),
		0x16 => Some('U'),
		0x17 => Some('I'),
		0x18 => Some('O'),
		0x19 => Some('P'),
		0x1A => Some('['),
		0x1B => Some(']'),
		0x1E => Some('A'),
		0x1F => Some('S'),
		0x20 => Some('D'),
		0x21 => Some('F'),
		0x22 => Some('G'),
		0x23 => Some('H'),
		0x24 => Some('J'),
		0x25 => Some('K'),
		0x26 => Some('L'),
		0x27 => Some(';'),
		0x28 => Some('\''),
		0x29 => Some('`'),
		0x2B => Some('\\'),
		0x2C => Some('Z'),
		0x2D => Some('X'),
		0x2E => Some('C'),
		0x2F => Some('V'),
		0x30 => Some('B'),
		0x31 => Some('N'),
		0x32 => Some('M'),
		0x33 => Some(','),
		0x34 => Some('.'),
		0x35 => Some('/'),
        
		0x39 => Some(' '),
        
		0x47 => Some('7'),
		0x48 => Some('8'),
		0x49 => Some('9'),
		0x4B => Some('4'),
		0x4C => Some('5'),
		0x4D => Some('6'),
		0x4F => Some('1'),
		0x50 => Some('2'),
		0x51 => Some('3'),
		0x52 => Some('0'),
		0x53 => Some('.'),
		0x4A => Some('-'),
		0x4E => Some('+'),
		0x37 => Some('*'),
        
		_ => None,
    };
	let Some(key) = key else {
		return;
	};
	let mut command = COMMAND.lock();
	if command.push(key).is_err() {
		command.clear();
		command.push(key).ok();
	}
	blinking();
}

