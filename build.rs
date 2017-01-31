use std::env;

fn main() {
    let profile = env::var("PROFILE").unwrap();
    if profile != "test" {
        // target triple
        let target
            = env::var("TARGET").unwrap();
        // extract the architecture name from the target triple
        let arch_name
            = target.split("-").next()
                    .expect("Couldn't parse target triple!");


        let boot_path = if arch_name == "x86_64" {
            format!("boot/target/x86_32-sos-bootstrap-gnu/{}/", profile)
        } else {
            panic!("target arch {} not yet supported, sorry!", arch_name);
        };
        println!("cargo:rustc-link-search=native={}", boot_path);
        println!("cargo:rustc-link-lib=static=boot");
    }


}
