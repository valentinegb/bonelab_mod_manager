use std::{env, path::PathBuf, process::Command};

fn main() {
    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

        Command::new("windres")
            .arg("icon.rc")
            .arg(out_dir.join("icon.lib"))
            .spawn()
            .unwrap();

        println!("cargo:rustc-link-search={}", out_dir.display());
        println!("cargo:rustc-link-lib=icon");
    }
}
