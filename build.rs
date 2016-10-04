extern crate nasm_rs;
use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let target = env::var("TARGET").unwrap();

    let arch_name = target.split("-").next()
                          .expect("Couldn't parse target triple!");

    let src_path = env::var("CARGO_MANIFEST_DIR")
                    .map(|path| PathBuf::from(path).join("src") )
                    .unwrap();

    let asm_dir = src_path.as_path()
                    .join("arch")
                    .join(arch_name);

    let asm_files = asm_dir.read_dir().unwrap()
                        .filter(|entry| match entry {
                            &Ok(ref file) =>
                                if let Some(ext) = file.path().extension() {
                                    "asm" == ext
                                } else {
                                    false
                                }
                          , &Err(_) => false
                        })
                        .map(|entry| entry.unwrap().path())
                        .collect::<Vec<PathBuf>>();

    nasm_rs::compile_library_args( "libboot.a"
                                 , asm_files.iter()
                                            .map(|path| path.to_str().unwrap())
                                            .collect::<Vec<&str>>()
                                            .as_slice()
                                 , &[ "-felf64" ]);

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=boot");

    for asm_file in asm_files {
        println!("cargo:rerun-if-changed={}", asm_file.to_str().unwrap());
    }

}
