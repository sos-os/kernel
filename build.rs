extern crate nasm_rs;
// use std::process::Command;
// use std::env;
// use std::path::Path;
//
// enum Arch { X86_64
//           , X86
//           , ArmV7
//           }
//
// impl Arch {
//
//     /// Convert an LLVM target triple into an `Arch`.
//     fn from_triple(triple: &str) -> Result<Arch, &str> {
//         let triple_arch_string = triple.split("-").next();
//
//         match triple_arch_string {
//             Some("x86_64") => Ok(Arch::X86_64)
//           , Some("x86") => Ok(Arch::X86)
//           , Some("armv7") => Ok(Arch::ArmV7)
//           , Some(thing_weird) =>
//                 Err(&format!("Got weird architecture: `{}`"), thing_weird)
//           , None =>
//                 Err(&format!("Target triple {} contained no valid arch part"
//                             , triple)
//
//         }
//     }
//
//     fn asm_extension(&self) -> &str {
//         match self {
//             &Arch::X86_64 => ".asm"
//           , &Arch::X86 => ".asm"
//           , &Arch::ArmV7 => unimplemented!()
//         }
//     }
//
//     fn nasm_flag(&self) -> &str {
//         match self {
//             &Arch::X86_64 => "-felf64"
//           , &Arch::X86 => "-felf32"
//           , &Arch::ArmV7 => unimplemented!()
//       }
//     }
//
//     fn arch_dir(&self) -> &str {
//         match self {
//             &Arch::X86_64 => "x86_64"
//           , &Arch::X86 => "x86"
//           , &Arch::ArmV7 => "armv7"
//       }
//     }
//
//     fn is_x86_64(&self) -> bool {
//         match self {
//             &Arch::X86_64 => true
//           , _ => false
//         }
//     }
//
//     fn is_x86_32(&self) -> bool {
//         match self {
//             &Arch::X86 => true
//           , _ => false
//         }
//     }
//
//     fn is_x86(&self) -> bool {
//         match self {
//             &Arch::X86 | &Arch::x86_64 => true
//           , _ => false
//         }
//     }
//
// }

fn main() {
    // let out_dir = Path::new(env::var("OUT_DIR").unwrap());
    //
    // let maybe_target = env::var("TARGET");
    //
    // let arch = maybe_target.map(Arch::from_triple).unwrap();
    // let arch_nasm_flags = arch.nasm_flag();
    //
    // let asm_dir = Path::new("src").join("arch")
    //                               .join(arch.arch_dir());
    //
    // let asm_ext = arch.asm_extension();
    //
    // fn nasm(filename: &str) -> Result<Command::ExitStatus, &str> {
    //     // TODO: can we refactor this to use the nasm-rs crate?
    //     let path = asm_dir.with_file_name(format!("{}{}", filename, asm_ext));
    //                       .to_str()?;
    //
    //     Command::new("nasm")
    //         .args(&[ path
    //                , arch_nasm_flags
    //                , "-o"
    //                , out_dir.with_file_name(format!("{}{}", filename, ".o"))
    //                         .to_str().unwrap()
    //                ])
    //         .status()
    //         .map(|ok| {
    //             println!("cargo:rustc-link-lib=static={}", filename);
    //             println!("cargo:rerun-if-changed={}", path);
    //             ok
    //         })
    // }
    //
    // nasm("boot").unwrap();
    //
    // if arch.is_x86_64() {
    //         nasm("start_64").unwrap();
    //         nasm("multiboot").unwrap();
    // };
    //
    // println!("cargo:rustc-link-search=native={}", out_dir);
    nasm_rs::compile_library_args( "libboot.a"
                                 , &[ "src/arch/x86_64/multiboot.asm"
                                    , "src/arch/x86_64/boot.asm" ]
                                 , &[ "-felf64" ]);
     println!("cargo:rustc-link-lib=static=boot");
    //  println!("cargo:rerun-if-changed=/src/asm/boot.asm");
}
