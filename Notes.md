Notes
=====

On designing a userland
-----------------------

+ provide user code written in Rust with a fluent Rust API
+ [capability-based security](https://en.wikipedia.org/wiki/Capability-based_security)

#### prior art
+ `ioctl` is a great example of _exactly what not to do_ - every system call should be its own call. requiring the programmer to keep track of both syscalls and values to pass to `ioctl` is a pain.
+ `kqueue` is good
