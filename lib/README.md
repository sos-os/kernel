# Libraries


### Rust libraries

In order to cross-compile SOS on some platforms, we need to build our own version of `libcore` with the correct `target.json` file. Therefore, we include phil-opp/nightly-libcore repository as a submodule. This version of Rust's core library has been patched to include a feature flag for disabling floating point code, allowing us to build `libcore` with SSE disabled.

The main `rust-lang/rust` repo has also been included as a submodule, as we also depend on `liballoc`, `libcollections`, and `librustc_unicode`. Unlike `libcore`, these libraries do not need to be patched, and we can build against the version from the main Rust repo.


### sos-alloc
This crate contains the SOS memory allocation library. It's included as a crate here so that it can be used in kernel- and user-space OS components. `sos-alloc` provides support (or, will eventually) for multiple allocation strategies, and can be used as a replacement for the Rust allocator if specified at compile-time.


### sos-intrusive

This library contains intrusive data structure implementations that are used in `sos-alloc`'s buddy heap implementation.

### sos-vga

This library contains code for interacting with a system's VGA buffer.

### sos-multiboot2

A library for accessing information from Multiboot 2.
