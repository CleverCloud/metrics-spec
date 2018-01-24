extern crate metrics_lib;

use std::mem;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void};

// In order to work with the memory we expose (de)allocation methods
#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut c_void;
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut c_void, cap: usize) {
    unsafe  {
        let _buf = Vec::from_raw_parts(ptr, 0, cap);
    }
}

#[no_mangle]
pub extern "C" fn parse_toml(data: *mut c_char) -> *mut c_char {
    unsafe {
        let data = CStr::from_ptr(data);
        let parsed = metrics_lib::parse_toml(data.to_str().unwrap()).unwrap();
        let yaml = metrics_lib::generate_yaml(&parsed).unwrap();
        let s = CString::new(yaml).unwrap();
        s.into_raw()
    }
}

fn main() {
    println!("noop");
}
