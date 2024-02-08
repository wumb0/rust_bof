#![no_std]

#[cfg(feature = "alloc")]
pub use bofalloc::ALLOCATOR;
pub use bofhelper::{bootstrap, BeaconPrintf, BofData, CALLBACK_ERROR};
pub use bofentry_macro::bof;