use core::arch::asm;

fn pulse(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
        );
    }
}

pub fn wake() {
    let divisor: u32 = 1193182 / 100;
        pulse(0x43, 0x36);
        pulse(0x40, (divisor & 0xff) as u8);
        pulse(0x40, (divisor >> 8) as u8);
}