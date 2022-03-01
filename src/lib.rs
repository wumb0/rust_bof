#![no_std]

extern crate alloc;
use bofhelper::{import_function, BofData, CALLBACK_OUTPUT, beacon_print};
use alloc::format;

use bofentry::bof_entry;

import_function!(KERNEL32!OutputDebugStringA(s: *const u8));

bof_entry!(entry);

fn entry(mut data: BofData) {
    unsafe { OutputDebugStringA("Hello world!\n\0".as_ptr()) };
    let s = data.get_str();
    let _int = data.get_int();
    let asdf = format!("my string = {}", s);
    beacon_print!("Hello world! {}", asdf);
    unsafe { OutputDebugStringA(asdf.as_ptr()) };
}