#![no_std]

extern crate alloc;
use alloc::format;
use bofhelper::{beacon_print, import_function, BofData, CALLBACK_OUTPUT};

use bofentry::bof_entry;

import_function!(KERNEL32!OutputDebugStringA(s: *const u8));

bof_entry!(entry);

fn entry(mut data: BofData) {
    unsafe { OutputDebugStringA("Hello world!\n\0".as_ptr()) };
    let s = data.get_str();
    let _int = data.get_int();
    let asdf = format!("my string = {}\0", s);
    beacon_print!("Hello world! {}", asdf);
    unsafe { OutputDebugStringA(asdf.as_ptr()) };
}
