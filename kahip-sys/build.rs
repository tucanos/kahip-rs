use std::env;
use std::path::PathBuf;

fn main() {
    let include_dir = match env::var("KAHIP_INCLUDE_DIR") {
        Ok(x) => PathBuf::from(x),
        Err(_) => match env::var("KAHIP_DIR") {
            Ok(x) => PathBuf::from(x).join("include"),
            Err(_) => PathBuf::from("/usr/include"),
        },
    };
    let kahip_h = include_dir
        .join("kaHIP_interface.h")
        .into_os_string()
        .into_string()
        .expect("Not an UTF-8 path");
    let lib_dir = match env::var("KAHIP_LIB_DIR") {
        Ok(x) => Some(PathBuf::from(x)),
        Err(_) => env::var("KAHIP_DIR")
            .map(|x| PathBuf::from(x).join("lib"))
            .ok(),
    };
    if let Some(lib_dir) = lib_dir {
        let lib_dir = lib_dir
            .into_os_string()
            .into_string()
            .expect("Not an UTF-8 path");
        println!("cargo:rustc-link-search={}", lib_dir);
        #[cfg(target_os = "macos")]
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir);
        #[cfg(target_os = "linux")]
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir);
        // non standard key
        // see https://doc.rust-lang.org/cargo/reference/build-script-examples.html#linking-to-system-libraries
        // and https://github.com/rust-lang/cargo/issues/5077
        println!("cargo:rpath={}", lib_dir);
    }
    println!("cargo:rerun-if-changed={}", kahip_h);
    println!("cargo:rustc-link-lib=kahip");

    bindgen::Builder::default()
        .header("stdbool.h")
        .header(kahip_h)
        .allowlist_function("kaffpa.*")
        .allowlist_function("process_mapping")
        .allowlist_function("node_separator")
        .allowlist_function("reduced_nd")
        .allowlist_function("edge_partitioning")
        .allowlist_var("FAST.*")
        .allowlist_var("ECO.*")
        .allowlist_var("STRONG.*")
        .allowlist_var("MAPMODE_.*")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("src/binding.rs")
        .expect("Couldn't write bindings!");
}
