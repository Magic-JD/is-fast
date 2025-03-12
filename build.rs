use clap::CommandFactory;
use clap_complete::{generate_to, shells::*};
use clap_mangen::Man;
use std::fs::File;
use std::io::Write;

include!("src/cli/command.rs");

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/cli/command.rs");

    let cmd = Cli::command();

    let mut buffer = Vec::new();
    Man::new(cmd)
        .render(&mut buffer)
        .expect("Failed to generate man page");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let man_path = std::path::Path::new(&out_dir).join("is-fast.1");

    let mut file = File::create(&man_path).expect("Failed to create man file");
    file.write_all(&buffer).expect("Failed to write man file");

    let mut cmd = Cli::command();
    generate_to(Bash, &mut cmd, "is-fast", &out_dir).unwrap();
    generate_to(Zsh, &mut cmd, "is-fast", &out_dir).unwrap();
    generate_to(Fish, &mut cmd, "is-fast", &out_dir).unwrap();
    generate_to(PowerShell, &mut cmd, "is-fast", &out_dir).unwrap();
}
