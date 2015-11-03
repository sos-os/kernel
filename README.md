SOS: the Stupid Operating System
--------------------------------

SOS is a simple, tiny toy OS implemented in Rust.

I'm writing this mostly for fun, to learn more about OS design and kernel hacking, so don't expect anything new or exciting out of this project.

Inspiration, and a reasonable amount of code, taken from Phil Oppermann's great [series of blog posts](http://blog.phil-opp.com/rust-os/) on the subject, Charlie Somerville's [rustboot](https://github.com/charliesome/rustboot), and Samy Pessé's [_How to Make an Operating System_](https://www.gitbook.com/book/samypesse/how-to-create-an-operating-system/details).

The Makefile currently expects that you're using cross-compiled GNU `binutils`, since I'm building on my Mac and the OS X linker obviously won't work. Eventually it'll be smarter about this.
