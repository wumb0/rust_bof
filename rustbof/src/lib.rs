#![no_std]

extern crate alloc;
use alloc::format;
use bofentry::bof;
use bofhelper::{beacon_print, import_function, BofData, CALLBACK_OUTPUT};

import_function!(KERNEL32!OutputDebugStringA(s: *const u8));

// you can specify the export name in the proc macro or just use it bare
// to have it use the function name!
#[bof(entrypoint)]
fn entry(mut data: BofData) {
    unsafe { OutputDebugStringA("Hello world!\n\0".as_ptr()) };
    let s = data.get_str();
    let _int = data.get_int();
    let asdf = format!("my string = {}\0", s);
    beacon_print!("Hello world! {}", asdf);
    unsafe { OutputDebugStringA(asdf.as_ptr()) };
}
