use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn bof(args: TokenStream, input: TokenStream) -> TokenStream {
    let syn::ItemFn { attrs, vis, sig, block } = syn::parse_macro_input!(input);
    let fn_ident = &sig.ident;
    let stmts = &block.stmts;
    let export_ident: syn::Ident = if args.is_empty() {
        fn_ident.clone()
    } else {
         syn::parse_macro_input!(args)
    };
    quote::quote! {
        #[no_mangle]
        unsafe extern "C" fn #export_ident(args: *mut u8, alen: i32) {
            let mut data = bofhelper::BofData::parse(args, alen);
            if bofhelper::bootstrap(data.get_data()).is_none() {
                bofhelper::BeaconPrintf(
                    bofhelper::CALLBACK_ERROR,
                    "BOF relocation bootstrap failed\0".as_ptr(),
                );
                return;
            }

            #[cfg(feature = "alloc")]
            bofalloc::ALLOCATOR.initialize();

            #(#attrs)* #vis #sig {
                #(#stmts)*
            }

            #fn_ident(data);

            #[cfg(feature = "alloc")]
            bofalloc::ALLOCATOR.destroy();
        }
    }.into()
}
