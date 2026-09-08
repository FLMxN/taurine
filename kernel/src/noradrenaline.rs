use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
    PageFaultErrorCode,
};
use pic8259::ChainedPics;
use spin::Mutex;

use crate::chill;

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[repr(u8)]
#[derive(Clone, Copy)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

pub fn brace() {
    unsafe {
        x86_64::instructions::interrupts::disable();
        let idt = &raw mut IDT;
        (*idt).breakpoint.set_handler_fn(breakpoint_handler);
        (*idt).page_fault.set_handler_fn(page_fault_handler);
        (&mut *idt)[InterruptIndex::Timer as u8].set_handler_fn(time_handler);
        (*idt).load();
        PICS.lock().initialize();
        x86_64::instructions::interrupts::enable();
    }
}

extern "x86-interrupt" fn breakpoint_handler(
    _stack_frame: InterruptStackFrame,
) {
    panic!("breakpoint exception");
}

extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: PageFaultErrorCode,
) {
    panic!("page fault exception");
}

extern "x86-interrupt" fn time_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS
            .lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
        chill();
    }
}