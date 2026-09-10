use core::arch::asm;

pub fn faint() -> ! {
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