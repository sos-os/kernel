# Stupid Operating System [![Build Status](https://travis-ci.org/hawkw/sos-kernel.svg?branch=master)](https://travis-ci.org/hawkw/sos-kernel) [![Dependency Status](https://dependencyci.com/github/hawkw/sos-kernel/badge)](https://dependencyci.com/github/hawkw/sos-kernel) [![MIT License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat)](https://github.com/hawkw/sos-kernel/LICENSE-MIT) [![Latest RustDoc](https://img.shields.io/badge/rustdoc-latest-orange.svg)](http://www.elizas.website/sos-kernel/sos_kernel/index.html) [![Gitter](https://img.shields.io/gitter/room/sos-os/sos-os.svg)](https://gitter.im/sos-os)

SOS is a simple, tiny toy OS implemented in Rust.

I'm writing this mostly for fun, to learn more about OS design and kernel hacking, so don't expect anything new or exciting out of this project.

Inspiration, and a reasonable amount of code, taken from @phil-opp's great [series of blog posts](http://os.phil-opp.com) on the subject, Charlie Somerville's [rustboot](https://github.com/charliesome/rustboot), and Samy Pessé's [_How to Make an Operating System_](https://www.gitbook.com/book/samypesse/how-to-create-an-operating-system/details).

design goals
------------

 + **POSIX compliance is not a goal** (though it would be cool)
 + **Hybrid/loosely microkernel** (i.e., move code to user space *when convenient/practical*)
 + Possibly provide the **Rust stdlib** at the OS level.
 + **JVM-style** memory allocation?
 + Possibly experiment with a **[Plan 9-esque](https://en.wikipedia.org/wiki/9P_(protocol)) networking stack** eventually?


building & running
------------------

I've included a simple [`Makefile`](Makefile) to automate building and running SOS. This README lists most of the important make targets, but there's also a `$ make help` command, which will print a list of all available targets.

### setting up your build environment
In order to build SOS, you'll need to properly configure your build environment. Since this process is fairly complex, I've provided some automatic installation shell scripts to make it a bit more painless.

+ `$ make env` will install and configure build dependencies

If you don't trust the scripts, or if you're curious to know what they're doing, you can also follow the manual install instructions in [`BUILDING.md`](BUILDING.md).

### building & running the OS
  + `$ make kernel` compiles & links the kernel binary
  + `$ make iso` makes the kernel and builds a bootable ISO image
  + `$ make run` compiles the kernel, makes the ISO, and boots QEMU from the ISO
