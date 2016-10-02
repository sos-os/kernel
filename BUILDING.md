setting up a dev environment
============================

installing Rust
---------------

In order to build SOS, you need an up-to-date version of the Rust compiler. SOS uses several gated features that can only be built on the nightly Rust release channel.

The suggested way to install Rust is using [Rustup](https://www.rustup.rs), a tool for managing multiple versions of Rust. If you don't have Rustup installed, you can install it by running this command in your terminal:
```
$ curl https://sh.rustup.rs -sSf | sh
```

Once Rustup is installed, run
```
$ rustup update nightly
```

to ensure the nightly release branch is up to date.

If you've set the stable or beta Rust release channels as the global default, you should run
```
$ rustup override nightly
```
in the SOS root directory, to set the nightly release channel as the default for SOS.

installing dev dependencies
---------------------------

Once you have Rust installed, you will need the following additional dependencies:
+ `nasm`
+ `ld`
+ `grub-mkrescue` & `xorriso`
+ `qemu` for running the kernel under emulation

Depending on your OS, you'll want to install these dependencies somewhat differently.

### linux

On Debian you can install them with

```
$ sudo apt-get install nasm xorriso qemu build-essential
```
On Arch Linux you can install them with
```
$ sudo pacman -S --needed binutils grub libisoburn nasm qemu
```
And on Fedora with
```
$ sudo dnf install nasm xorriso qemu
```

### macOS

Installing dev dependencies on macOS is slightly trickier, as you will also need a cross-compiled version of GNU `binutils` to build SOS. Cross-compiling `binutils` will require some additional dependencies.

You can install a majority of dependencies using the [Homebrew](https://github.com/Homebrew/brew) package manager. I've included a `Brewfile` for automatically installing these dependencies. To use the `Brewfile`, run the following sequence of commands:

```
$ brew update
$ brew tap Homebrew/bundle
$ brew bundle
```

Once you've installed the `Brewfile`, you'll need to cross-compile `binutils`. Right now, the best way to go about that is to run [this shell script](http://intermezzos.github.io/book/appendix/osx-install.html) by Steve Klabnik. I'm working on further automating the dev environment setup process on OS X, so this may get easier soon.

Once the script is complete, add the following to `~/.cargo/config`:
```yaml
[target.x86_64-unknown-sos-gnu]
linker = "/Users/yourusername/opt/bin/x86_64-pc-elf-gcc"
```

### windows
Seriously?
Windows isn't supported; I can't possibly advise it.

installing `xargo`
-----------------

Once you've installed Rust and the dev dependencies, you'll need to install [`xargo`](https://github.com/japaric/xargo), a tool for cross-compiling Rust programs. You can install `xargo` quite easily by running
```
$ cargo install xargo
```
