use core::ffi::c_void;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(
    dest: *mut c_void,
    value: i32,
    count: usize,
) -> *mut c_void {
    for i in 0..count {
        unsafe {
            let dest = dest as *mut u8;
            dest.add(i).write(value as u8);
        }
    }

    dest as *mut c_void
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(
    dest: *mut c_void,
    src: *const c_void,
    count: usize,
) -> *mut c_void {
    for i in 0..count {
        unsafe {
            let dest = dest as *mut u8;
            let src = src as *const u8;
            dest.add(i).write(src.add(i).read());
        }
    }

    dest as *mut c_void
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(
    a: *const c_void,
    b: *const c_void,
    count: usize,
) -> i32 {
    for i in 0..count {
        let x = unsafe { a.add(i).read() };
        let y = unsafe { b.add(i).read() };

        let x = x as u8;
        let y = y as u8;
        
        if x != y {
            return x as i32 - y as i32;
        }
    }

    0
}