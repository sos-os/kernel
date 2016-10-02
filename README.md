#Stupid Operating System [![Build Status](https://travis-ci.org/hawkw/sos-kernel.svg?branch=master)](https://travis-ci.org/hawkw/sos-kernel) [![MIT License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat)](https://github.com/hawkw/sos-kernel/LICENSE-MIT) [![Latest RustDoc](https://img.shields.io/badge/rustdoc-latest-orange.svg)](https://hawkweisman.me/sos-kernel)



SOS is a simple, tiny toy OS implemented in Rust.

I'm writing this mostly for fun, to learn more about OS design and kernel hacking, so don't expect anything new or exciting out of this project.

Inspiration, and a reasonable amount of code, taken from @phil-opp's great [series of blog posts](http://os.phil-opp.com) on the subject, Charlie Somerville's [rustboot](https://github.com/charliesome/rustboot), and Samy Pessé's [_How to Make an Operating System_](https://www.gitbook.com/book/samypesse/how-to-create-an-operating-system/details).

Design goals
------------

 + **POSIX compliance is not a goal** (though it would be cool)
 + **Hybrid/loosely microkernel** (i.e., move code to user space *when convenient/practical*)
 + Possibly provide the Rust stdlib at the OS level.
 + JVM-style memory allocation?
 + Possibly experiment with a [Plan 9-esque](https://en.wikipedia.org/wiki/9P_(protocol)) networking stack eventually?


Building & Running
------------------

#### setting up your development environment
In order to build SOS, you'll need to properly configure your development environment. I'm working on including a shell script to automate the setup process, but for now, you'll need to follow the instructions in BUILDING.md.

#### building & running the OS
  + `$ make kernel` compiles & links the kernel binary
  + `$ make iso` makes the kernel and builds a bootable ISO image
  + `$ make run` compiles the kernel, makes the ISO, and boots QEMU from the ISO
