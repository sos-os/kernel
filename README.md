SOS: the Stupid Operating System
--------------------------------

SOS is a simple, tiny toy OS implemented in Rust.

I'm writing this mostly for fun, to learn more about OS design and kernel hacking, so don't expect anything new or exciting out of this project.

Inspiration, and a reasonable amount of code, taken from @phil-opp's great [series of blog posts](http://os.phil-opp.com) on the subject, Charlie Somerville's [rustboot](https://github.com/charliesome/rustboot), and Samy Pessé's [_How to Make an Operating System_](https://www.gitbook.com/book/samypesse/how-to-create-an-operating-system/details).

The Makefile currently expects that you're using cross-compiled GNU `binutils`, since I'm building on my Mac and the OS X linker obviously won't work. Eventually it'll be smarter about this.

### Design goals

 + **POSIX compliance is not a goal** (though it would be cool)
 + **Hybrid/loosely microkernel** (i.e., move code to user space *when convenient/practical*)
 + Possibly provide the Rust stdlib at the OS level.
 + JVM-style memory allocation?
 + Possibly experiment with a [Plan 9-esque](https://en.wikipedia.org/wiki/9P_(protocol)) networking stack eventually?


### Using Vagrant

For development on Mac OS, I'm using a Vagrant config written by @ashleygwilliams (see her repo [here](https://github.com/ashleygwilliams/x86-kernel)).

To run using vagrant (from the repo root directory):

 + `$ vagrant up`
 + `$ vagrant ssh -- -Y`
 + `$ multirust default nightly-2015-11-19`
 + `$ cd /vagrant`
 + `$ make run`
