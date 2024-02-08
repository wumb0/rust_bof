#![no_std]

#[cfg(feature = "alloc")]
pub use bofalloc::ALLOCATOR;
pub use bofhelper::{bootstrap, BeaconPrintf, BofData, CALLBACK_ERROR};

// helper function for defining an entrypoint easily
#[macro_export]
macro_rules! bof_entry {
    ($entry:ident) => {
        #[no_mangle]
        unsafe extern "C" fn entrypoint(args: *mut u8, alen: i32) {
            let mut data = $crate::BofData::parse(args, alen);
            if $crate::bootstrap(data.get_data()).is_none() {
                $crate::BeaconPrintf(
                    $crate::CALLBACK_ERROR,
                    "BOF relocation bootstrap failed\0".as_ptr(),
                );
                return;
            }

            #[cfg(feature = "alloc")]
            $crate::ALLOCATOR.initialize();

            $entry(data);

            #[cfg(feature = "alloc")]
            $crate::ALLOCATOR.destroy();
        }
    };
}
