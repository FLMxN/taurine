use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
    PageFaultErrorCode,
};

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init_idt() {
    unsafe {
        let idt = &raw mut IDT;
        (*idt).breakpoint.set_handler_fn(breakpoint_handler);
        (*idt).page_fault.set_handler_fn(page_fault_handler);
        (*idt).load();
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