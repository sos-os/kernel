extern crate nasm_rs;

use std::env;
use std::path::{PathBuf,  Path};
use std::process::{Command, ExitStatus, Output};
use std::io;
use std::io::{BufReader, BufRead, Write};

fn main() {

    if env::var("PROFILE").unwrap() != "test" {
        let out_dir // the output directory for compiled binaries
            = env::var("OUT_DIR").unwrap();

        let pwd = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let src_path // path to the source code directory
            = pwd.join("src");

        let target // target triple
            = env::var("TARGET").unwrap();

        let arch_name // extract the architecture name from the target triple
            = target.split("-").next()
                    .expect("Couldn't parse target triple!");

        // let asm_dir // construct the path to the arch-specific source directory...
        //     = src_path.as_path()        //...by concatenating the source path...
        //               .join("arch")     // ...the "arch" dir...
        //               .join(arch_name); //...and the name of the arch we're building
        //
        // let asm_files // get the name of each assembly file in the arch dir
        //     = asm_dir.read_dir().unwrap()
        //              .filter(|entry| match entry {
        //                     &Ok(ref file) =>
        //                         if let Some(ext) = file.path().extension() {
        //                             "asm" == ext
        //                         } else {
        //                             false
        //                         }
        //                   , &Err(_) => false
        //                 })
        //              .map(|entry| entry.unwrap().path())
        //              .collect::<Vec<PathBuf>>();
        //
        // // invoke NASM to compile and archive the assembly files
        // nasm_rs::compile_library_args( "libboot.a"
        //                              , // convert asm paths into &str
        //                                 asm_files.iter()
        //                                          .map(|path| path.to_str().unwrap())
        //                                          .collect::<Vec<&str>>()
        //                                          .as_slice()
        //                              , // TODO: determine this from target triple
        //                                &[ "-felf64" ]);
        //
        // println!("cargo:rustc-link-search=native={}", out_dir);
        // println!("cargo:rustc-link-lib=static=boot");
        //
        // xargo rustc --target x86_32-sos-bootstrap-gnu -- --emit=obj
        //
        // fn run(cmd: &mut Command) -> io::Result<ExitStatus> {
        //     use std::process::Stdio;
        //
        //     println!("running: {:?}", cmd);
        //     match cmd.stderr(Stdio::piped()).stdout(Stdio::piped())
        //               .spawn() {
        //         Ok(mut child) => {
        //             let stderr = BufReader::new(child.stderr.take().unwrap());
        //             for line in stderr.split(b'\n').filter_map(|l| l.ok()) {
        //                 print!("cargo:warning=");
        //                 std::io::stdout().write_all(&line).unwrap();
        //                 println!("");
        //             }
        //             child.wait()
        //         }
        //         Err(e) => Err(e),
        //     }
        // }
        // fn compile_bootstrap( path: &Path )
        //                     -> io::Result<ExitStatus> {
        //     // use std::error::Error;
        //     let out_dir = PathBuf::from("target");
        //     run(Command::new("xargo rustc")
        //         // .arg("rustc")
        //         .arg("--target")
        //         .arg("x86_32-sos-bootstrap-gnu")
        //         .arg("--")
        //         .arg("--emit=obj=target/boot.o")
        //         .current_dir(std::env::current_dir()?.join("bootstrap").as_path()))
        //
        // }

        use std::process::Command;
        use std::process::Stdio;

        let boot_path = if arch_name == "x86_64" {


            "boot/target/"

        } else {
            panic!("target arch {} not yet supported, sorry!", arch_name);
        };
        // println!( "cargo:rustc-link-search=native={}", boot_path);
        // println!("cargo:rustc-link-lib=static=boot");

        // // for each assembly file detected, tell cargo to re-run
        // // if that file has changed
        // for asm_file in asm_files {
        //     println!("cargo:rerun-if-changed={}", asm_file.to_str().unwrap());
        // }
    }


}
