# Libraries


### rust 
In order to cross-compile SOS on some platforms, we need to build our own version of `libcore` with the correct `target.json` file. Therefore, we include the `rust-lang/rust` repository as a submodule.


### sos-alloc
This crate contains the SOS memory allocation library. It's included as a crate here so that it can be used in kernel- and user-space OS components. `sos-alloc` provides support (or, will eventually) for multiple allocation strategies, and can be used as a replacement for the Rust allocator if specified at compile-time.
