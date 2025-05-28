use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use walkdir::WalkDir;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command;

fn main() {
    let resources = Path::new("../../resources");
    let out_dir = resources.join("out");
    let resourcepack_dir = resources.join("resourcepack");
    let resourcepack_out = "resourcepack.zip";

    println!("cargo:rerun-if-changed={}", input_dir);
    for entry in WalkDir::new(input_dir) {
        let entry = entry.unwrap();
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }

    if !out_dir.exists() {
        fs::create_dir_all(&out_dir)
            .expect("Failed to create output directory");
    }

    let resourcepack_out_dir = out_dir.join(resourcepack_name);

    let status = Command::new("zip")
        .args(&[
            "-r",
            output_zip.to_str().unwrap(),
            ".",
        ])
        .current_dir(&resourcepack_dir)
        .status()
        .expect("Failed to execute `zip` command");

    if !status.success() {
        panic!("`zip` exited with status: {}", status);
    }
}