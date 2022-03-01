Rust BOFs for Cobalt Strike
===========================
This took me like 4 days, but I got it working... rust core + alloc for Cobalt Strike BOFs.  
This is very much a PoC, but I'd love to see others playing around with it and contributing.  

Building
========
1. Install mingw  
2. Install nightly rust with the x86_64-pc-windows-gnu and i686-pc-windows-gnu toolchains  
3. Run `cargo install cargo-make`  
4. Run `cargo make`  
5. ????  
6. BOFit  

Make your own
=============
Edit the entry function in rustbof/src/lib.rs. You can add new args by using the `bof_pack` function in the aggressor script, just don't change the first two because those are the relocations.  

How the fk
==========
I feel like I want to write a blog post about it at some point, but for now, here was the process:  
- How do I compile object files from rust?  
  - rustc has an `--emit=obj` flag that will just emit the object files into the deps folder of the target  
- A BOF is a **single** object file, not many. Rust compiles each component into its own .o file. How do I combine them?  
  - ld has a feature called "relocatable" (-i) which is used for incremental linking  
- The Cobalt Strike loader throws throwing a NullPointerException on `beacon_inline_execute`. Why?  
  - Some decompilation and bytecode debugging led me to find that the CS COFF parser was choking on some symbols in the object. Turns out those symbols were just included as debug (file) info.  
  - In my investigation I found that the `OBJExecutable` and `OBJParser` classes in cobaltstrike.jar have main functions that take the path to an object file and print a bunch of useful information!  
- How do I remove uneeded symbols in an object file?  
  - Well, `strip` works! `strip` actually has a `--strip-uneeded` flag that strips everything not needed for relocations, like debug info!  
- Now the Cobalt Strike BOF loader complains about some undefined symbols like `rust_oom` and `__rust_alloc`. At this point I wasn't using alloc at all, but it was still being compiled and then added to the BOF via `ld -i`. How can I get rid of these symbols?  
  - At first I just removed the object file for alloc, since I wasn't using it. Easy.  
  - I think at one point I discovered the `--gc-sections` flag for ld, which allows you to define a root symbol via the `-u` flag and then it gets rid of any symbols that aren't ever referenced. That also fixes it.  
- OK. At this point we have something that loads up and does not crash. Let's try to import a function from KERNEL32. How do we make rust use `__imp_` symbols?  
  - We can define the link name via `#[link_name = "__imp_KERNEL32$OutputDebugStringA]`  
- The issue with the above is that the `__imp_` symbols are supposed to be pointers to the import table and not functions themselves, so rust thinks that the symbol is a single pointer to a function and not a double pointer to a function. How the heck do we convince rust that a function pointer is actually a function pointer pointer in a clean way?  
  - Well, it turns out we can use unsafe rust to make a pointer into a double pointer, but that's only half of the solution because I don't want to have to say `unsafe { make my function pointer a double pointer}(args)` every time I want to make a call.  
  - It is acutally possible to make rust import symbols via the `__imp_` method, but I was only able to get it working on variables and not functions, so it was kind of useless.  
- How can I automatically cast the pointer on call?  
  - If you didn't know, rust calls `deref` when a type is called. So you can wrap the function pointer in a type and then implement `core::ops::Deref` for that type to cast the pointer to a double pointer on the fly. The result is a successful function call.  
- So now we can import functions, but now what about an allocator? I want to be able to use strings and vectors. That's the nice part of rust core, alloc is easy to implement! The allocator just creates and manages a heap via NTDLL's Rtl*Heap functions. I defined a global allocator that must be initialized to be used. Now I'm getting those undefined symbols from alloc again such as `__rust_alloc` and `rust_oom`. Why?  
  - It turns out when rust links a binary it creates those symbols and points them to either the system allocator or a custom allocator if one is defined. I need to define them manually.  
- Now I'm getting an undefined symbol that looks like the name of my global allocator. What is happening?  
  - BOFs don't use the BSS and the allocator was getting put in the BSS. I just explicitly told rust to put the ALLOCATOR inside the .data section via `#[link_section = ".data"]`. Fixed.  
- Now that I can allocate memory, let's try using the `format!` macro in `alloc` to make an allocated `String`. It crashes! What gives?  
  - This one took me a while to track down. The BOF loader does not relocate things in .data and .rdata, only .text. The COFF parser has a function at `pe.OBJExecutable.getRelocations` that creates a relocation structure for symbols in .text, but nothing else.  
  - The crash was happening in an virtual function table in .rdata that was not getting updated.  
  - Regardless, the BOF loader doesn't support type 1 (IMAGE_REL_AMD64_ADDR64), which is what rust was generating for .rdata.   
- How can I get the relocations applied?  
  - Write my own bootstrapper to manually apply the relocations before I do anything crazy like vtable references  
  - Not that hard, but requires some coordination from the Cobalt Strike client side  
  - I used Cobalt Strike's `OBJExecutable` class to parse the COFF and the `Parser` class to pack in the extra relocations. This is pretty much what CS does in `getRelocations`. The rust side gets the info via the BOF arguments and then applies the relocations.  
- How do I know where the beacon is putting the other sections?  
  - The BOF loader in beacon is going to choose some place to put the `.data` and `.rdata` sections, but we don't know where those are from our code.  
  - I ended up adding importable symbols at the beginning of the .text, .rdata, and .data sections via modifying the linker script. If anyone has better ideas here I'd love to hear them.  
- When those symbols were imported as extern static usize variables one of them generates an undefined `refptr` symbol. How do I stop it from doing that?  
  - I ended up importing them as functions and then casting those to usize. A hack, but it gets the job done.  
  - After fixups, vtable calls work!  
- On to 32-bit. When you try to load it a bunch of syms are undefined, including a bunch of stuff starting with `__`. How can we resolve that???  
  - 32-bit adds an underscore to a bunch of stuff for some reason I could look up but don't care.  
  - I changed the import symbol to be `_imp_` instead of `__imp_`  
    - Doing this broke 64 bit so I added a `cfg_attr` to make the import name have the correct number of underscores depending on the target  
  - I added a `cfg_attr` to the `__section_start__` symbols from the linker to use a link name minus one underscore for x86 only.  
  - The final symbol is the dreaded `chkstk`... which I've dealt with at length  
- How can we get chkstk?  
  - It is defined in NTDLL, so we can just import `NTDLL!_chkstk` and define a `__chkstk` ourselves that calls it  

yeah and now both 32 and 64 bit BOFs work.  
I haven't tried anything too incredibly fancy yet, but let me know if there are issues.  

Versions
========
I don't know if this will work with future versions of CS or rust. Maybe.  
I've been using rust 1.61.0 nightly and CS 4.5.  

Known issues
============
TBD I guess.  

TODO
====
- Make the bofhelper library more robust  
- Docs, always \<3  
- Add some `Result` action with proper error types  

Thanks
======
- TrustedSec's COFF parser really helped with this - https://github.com/trustedsec/COFFLoader  
- Delorie COFF structure pages - https://delorie.com/djgpp/doc/coff  
- This post on Rust intermediates - https://medium.com/@squanderingtime/manually-linking-rust-binaries-to-support-out-of-tree-llvm-passes-8776b1d037a4  
- This post on debugging java bytecode - https://www.crowdstrike.com/blog/native-java-bytecode-debugging-without-source-code/  
- Sleep & CS beacon docs  