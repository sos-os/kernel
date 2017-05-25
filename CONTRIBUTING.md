Contributing to SOS
===================

**Looking for a first issue?** You might want to start out by looking at [issues tagged "easy"](https://github.com/hawkw/sos-kernel/issues?q=is%3Aissue+is%3Aopen+label%3Aeasy). These are issues that, while important, will probably require less knowledge of Rust, systems programming in general, or SOS, and might make good jumping-off points for potential contibutors.

### Table of Contents

+ [What do I need to know before contributing?](#what-do-i-need-to-know-before-contributing)
    - [Code of Conduct](#code-of-conduct)
    - [Licensing](#licensing)
    - [Setting Up a Dev Environment](#setting-up-a-dev-environment)
+ [Project Goals & Objectives](#project-goals--objectives)
+ [Conventions & Style Guides](#conventions--style-guides)
    - [Git Conventions](#git-conventions)
        * [Pull Requests](#pull-requests)
        * [Commit Messages](#commit-messages)
    - [Coding Style](#coding-style)
        * [Tools to assist with coding style](#tools-to-assist-with-coding-style)

What do I need to know before contributing?
===========================================

### Code of Conduct

This project adheres to the Contributor Covenant [code of conduct](CODE_OF_CONDUCT.md).
By participating, you are expected to uphold this code.
Please report unacceptable behavior to [eliza@elizas.website](mailto:eliza@elizas.website).

### Licensing

SOS is dual-licensed under the [MIT](LICENSE-MIT) and [Apache 2](LICENSE-APACHE) open-source licenses. By contributing code to SOS, you agree to waive all copyright claims on your contribution and allow it to be distributed under these licenses.

### Setting Up a Dev Environment

Building an OS is often a fairly difficult process, and can require a number of specific tools, libraries, and other dependencies installed and configured on the host system. In order to make contributing to SOS as easy as possible, we've tried to streamline the development environment setup process as much as possible, but there are still a few steps required before you can build SOS. Please see [BUILDING.md](BUILDING.md) for detailed instructions on how to build SOS.

In addition, the [tools to assist with coding style](#tools-to-assist-with-coding-style) section in this document provides information on optional tools that can be used to ensure your contributions conform to SOS' preferred coding style.

Project Goals & Objectives
==========================

+ Minimise assembly language code
    + Ideally, files ending in `.asm` should be used ONLY for the boot sequence
    + When platform-specific assembly code is required after the boot sequence, it should be placed in [inline assembly](https://doc.rust-lang.org/book/inline-assembly.html) in Rust functions, not in `.asm` files.
    + This has the following advantages:
        + It requires less linker configuration and makes building SOS much less of a pain.
        + It makes it much easier for Rust code to call code written in assembly language without mucking around with FFI.
        + It encourages us to write as much code as possible in a safe, high-level language, and write assembly only when it is strictly required.
+ Eventually, be able to boot on x86, x86_64, and ARMv7 machines.
+ Move as much out of kernel space as seems reasonable.


Conventions & Style Guides
==========================

Git Conventions
---------------

### Pull requests

In order to be accepted and merged, a pull request must meet the following conditions.

##### Pull requests MUST

+ Build successfully on [Travis](https://travis-ci.org/hawkw/sos-kernel)
+ Include RustDoc comments for any public-facing API functions or types
+ Include tests for any added features
+ Reference any closed issues with the text "Closes #XX" or "Fixes #XX" in the pull request description

##### Pull requests MUST NOT

+ Include any failing tests
+ Decrease overall project test coverage
+ Have any outstanding changes requested by a reviewer.

### Commit messages

Commit messages should follow the [Angular.js Commit Message Conventions](https://github.com/conventional-changelog/conventional-changelog/blob/a5505865ff3dd710cf757f50530e73ef0ca641da/conventions/angular.md). We use [`clog`](https://github.com/clog-tool/clog-cli) for automatically generating changelogs, and commit messages must be in a format that `clog` can parse.

It is recommended that contributors read the linked documentation for the Angular commit message convention in full –– it's not that long. For the impatient, here are some of the most important guidelines:

##### Commit messages MUST

+ Be in present tense
+ Follow the form `<type>(<scope>): <subject>`
    + where `<type>` is one of:
        * **feat**: A new feature
        * **fix**: A bug fix
        * **docs**: Documentation only changes
        * **style**: Changes that do not affect the meaning of the code (white-space, formatting, missing
        semi-colons, etc)
        * **refactor**: A code change that neither fixes a bug or adds a feature
        * **perf**: A code change that improves performance
        * **test**: Adding missing tests
        * **chore**: Changes to the build process or auxiliary tools and libraries such as documentation
        generation
    + and `<scope>` (optionally) specifies the specific element or component of the project that was changed.

##### Commit messages MUST NOT

+ Include lines exceeding 100 characters

##### Commit messages MAY

+ Include the text `[skip ci]` if changing non-Rustdoc documentation.
    + This will cause Travis CI to skip building that commit.
    + Commits which change RustDoc documentation in `.rs` source code files should still be built on CI -- `[skip ci]` should only be used for commits which change external documentation files such as `README.md`
    + Commits which change configuration files for tools not used by Travis may also skip the CI build, at the discretion of the committer.


Code Style
----------

Rust code should:
+ Follow the [Rust style guidelines](https://github.com/rust-lang/rust/tree/master/src/doc/style/style) and the guidelines in the ["Effective Rust" section](https://doc.rust-lang.org/book/effective-rust.html) of the Rust Book,  except when contradicted by this document.
    + In particular, it should...
        + ...be indented with 4 spaces
        + ...not end files with trailing whitespace
        + ...follow the [Rust naming conventions](https://github.com/rust-lang/rust/tree/master/src/doc/style/style/)
    + An `.editorconfig` file is available for [compatible text editors](http://editorconfig.org/#download).
+ Use [comma-first style](https://gist.github.com/isaacs/357981) for all comma-delimited constructs.
+ Not exceed 80 characters per line.

The following deviations from the style guide are permitted:
+ [Comma-first style](https://gist.github.com/isaacs/357981) _may_ be used for all comma-delimited constructs. For example:

    ```rust
    let a_list = [ a
                 , b
                 , c
                 ];
    ```

    and

    ```rust
    let a_list = [ a, b, c, d
                 , e, f, g, h
                 ];
    ```
    are considered good style.

+ When wrapping `where` clauses, place them at the same indentation level as the corresponding `fn` or `impl` statement. For example:
    ```rust
    // Considered good style
    fn foo<A>(a: A) where A: Something {
        ...
    }
    ```
    and
    ```rust
    // Considered good style
    fn bar<A, B>(a: A) -> B
    where A: Something
        , B: Something + SomethingElse {
        ...
    }
    ```
    are considered good style, while
    ```rust
    // NOT considered good style
    fn baz<A, B>(a: A) -> B
        where A: Something
            , B: SomethingElse {
                ...
            }
    ```
    is not.


### Tools to Assist With Coding Style

#### EditorConfig

An [`.editorconfig` file](.editorconfig) is available for [compatible text editors](http://editorconfig.org/#download). If the EditorConfig plugin is installed in your text editor, it will use this file to automatically configure certain formatting settings for the `an-editor` repository.

#### rustfmt

[`rustfmt`](https://github.com/rust-lang-nursery/rustfmt) is a tool for automatically formatting Rust source code according to style guidelines. This repository provides a [`rustfmt.toml`](rustfmt.toml) file for automatically configuring `rustfmt` to use our style guidelines.

`rustfmt` may be installed by running

```bash
$ cargo install rustfmt
```

and invoked on a crate by running

```bash
$ cargo fmt
```

Additionally, there are `rustfmt` plugins [available](https://github.com/rust-lang-nursery/rustfmt#running-rustfmt-from-your-editor) for many popular editors and IDEs.

`rustfmt` may also be added as a [git pre-commit hook](https://git-scm.com/book/uz/v2/Customizing-Git-Git-Hooks) to ensure that all commits conform to the style guidelines.
