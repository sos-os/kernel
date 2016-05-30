Contributing to SOS
===================

Code Style
----------

Rust code should:
+ Follow the [Rust style guidelines](https://github.com/rust-lang/rust/tree/master/src/doc/style/style) except when contradicted by this document.
    + In particular, it should...
        + ...be indented with 4 spaces
        + ...not end files with trailing whitespace
        + ...follow the [Rust naming conventions](https://github.com/rust-lang/rust/tree/master/src/doc/style/style/)
    + An `.editorconfig` file is available for [compatible text editors](http://editorconfig.org/#download).
+ Use [comma-first style](https://gist.github.com/isaacs/357981) for all comma-delimited constructs.
+ Not exceed 80 characters per line.
+ When wrapping `where` clauses, place them at the same indentation level as the corresponding `fn` or `impl` statement. For example:
```rust
// Considered good style
fn foo<A>(a: A) where A: Something {
    ...
}

// Considered good style
fn bar<A, B>(a: A) -> B
where A: Something
    , B: Something + SomethingElse {
    ...
}

// NOT considered good style
fn baz<A, B>(a: A) -> B
    where A: Something
        , B: SomethingElse {
            ...
        }
```


Project Goals/Objectives
------------------------

+ Minimise assembly language code
    + Ideally, files ending in `.asm` should be used ONLY for the boot sequence
    + When platform-specific assembly code is required after the boot sequence, it should be placed in [inline assembly](https://doc.rust-lang.org/book/inline-assembly.html) in Rust functions, not in `.asm` files.
    + This has the following advantages:
        + It requires less linker configuration and makes building SOS much less of a pain.
        + It makes it much easier for Rust code to call code written in assembly language without mucking around with FFI.
        + It encourages us to write as much code as possible in a safe, high-level language, and write assembly only when it is strictly required.
+ Eventually, be able to boot on x86, x86_64, and ARMv7 machines.
+ Move as much out of kernel space as seems reasonable.
