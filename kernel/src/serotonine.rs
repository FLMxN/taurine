use core::arch::asm;
use heapless::Deque;
use spin::Mutex;

const COM1: u16 = 0x3F8;
const DATA: u16 = COM1;
const INTERRUPT_ENABLE: u16 = COM1 + 1;
const FIFO_CONTROL: u16 = COM1 + 2;
const LINE_CONTROL: u16 = COM1 + 3;
const MODEM_CONTROL: u16 = COM1 + 4;
const LINE_STATUS: u16 = COM1 + 5;

static RECEIVED: Mutex<Deque<u8, 256>> = Mutex::new(Deque::new());
static TRANSMIT: Mutex<Deque<u8, 256>> = Mutex::new(Deque::new());

unsafe fn write_port(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
    }
}

unsafe fn read_port(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack));
    }
    value
}

pub fn initialize() {
    unsafe {
        write_port(INTERRUPT_ENABLE, 0x00);
        write_port(LINE_CONTROL, 0x80);
        write_port(DATA, 0x01);
        write_port(INTERRUPT_ENABLE, 0x00);
        write_port(LINE_CONTROL, 0x03);
        write_port(FIFO_CONTROL, 0xC7);
        write_port(MODEM_CONTROL, 0x0B);
        write_port(INTERRUPT_ENABLE, 0x00);
    }
    send_bytes(b"TAURINE_UP\n");
}

pub fn send(byte: u8) {
    TRANSMIT.lock().push_back(byte).ok();
}

pub fn send_bytes(bytes: &[u8]) {
    for &byte in bytes {
        send(byte);
    }
}

pub fn flush() {
    loop {
        unsafe {
            if read_port(LINE_STATUS) & 0x20 == 0 {
                return;
            }
        }

        let Some(byte) = TRANSMIT.lock().pop_front() else {
            return;
        };
        unsafe {
            write_port(DATA, byte);
        }
    }
}

pub fn try_receive() -> Option<u8> {
    RECEIVED.lock().pop_front()
}

pub fn poll_receive() {
    unsafe {
        while read_port(LINE_STATUS) & 0x01 != 0 {
            let byte = read_port(DATA);
            RECEIVED.lock().push_back(byte).ok();
        }
    }
}