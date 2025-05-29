use std::{env, fs, process::Command};
use std::path::PathBuf;
use walkdir::WalkDir;
use sha1::Digest;

fn main() {
    let resource_dir_name = "../../resources";
    let gradle_project_dir_name = "../../Thecrown";
    let resourcepack_dir = "resourcepack";
    let resourcepack_gen_name = "generated_resourcepack";
    let resourcepack_zip_name = "resourcepack.zip";
    let output_dir_name = "output";
    let models_dir_name = "models";
    let bbmodel_dir_name = "bbmodel";
    let model_mapping_name = "model_mapping.json";

    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let resources = manifest_dir.join(resource_dir_name);
    let gradle_project_dir = manifest_dir.join(gradle_project_dir_name);

    let resourcepack_dir = resources.join(resourcepack_dir);
    let out_dir = resources.join(output_dir_name);
    let bbmodel_dir = resources.join(bbmodel_dir_name);
    let resourcepack_gen_dir = out_dir.join(resourcepack_gen_name);
    let models_dir = out_dir.join(models_dir_name);
    let mappings_path = resources.join(model_mapping_name);

    if resourcepack_dir.exists() {
        // Cleanse directories
        if out_dir.exists() {
            fs::remove_dir_all(&out_dir).expect("Failed to remove existing output directory");
        }
        fs::create_dir_all(&out_dir).expect("Failed to create output directory");
        fs::create_dir_all(&resourcepack_gen_dir).expect("Failed to create generated resourcepack directory");
        fs::create_dir_all(&models_dir).expect("Failed to create models directory");

        // Build script rerun-if-changed
        println!("cargo:rerun-if-changed={}", out_dir.display());
        for entry in WalkDir::new(&resourcepack_dir) {
            let entry = entry.expect("WalkDir error");
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
        for entry in WalkDir::new(&bbmodel_dir) {
            let entry = entry.expect("WalkDir error");
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }

        // Copy resourcepack to generated_resourcepack
        for entry in WalkDir::new(&resourcepack_dir) {
            let entry = entry.expect("WalkDir error");
            let rel_path = entry.path().strip_prefix(&resourcepack_dir).expect("Failed to strip prefix");
            let dest_path = resourcepack_gen_dir.join(rel_path);
            if entry.file_type().is_dir() {
                fs::create_dir_all(&dest_path).expect("Failed to create directory");
            } else {
                fs::copy(entry.path(), &dest_path).expect("Failed to copy file");
            }
        }

        // Now call PackBuilder.generate. We need to call the Java program using gradle.
        let gradlew = if cfg!(windows) { "gradlew.bat" } else { "./gradlew" };
        let status = Command::new(gradlew)
            .current_dir(&gradle_project_dir)
            .args(&[
                "generatePack",
                &format!("-PbbmodelDir={}", bbmodel_dir.display()),
                &format!("-PrespackDir={}", resourcepack_gen_dir.display()),
                &format!("-PmodelsDir={}", models_dir.display()),
                &format!("-Pmappings={}", mappings_path.display()),
            ])
            .status()
            .expect("Failed to run Gradle generatePack task");
        if !status.success() {
            panic!("Gradle generatePack failed: {}", status);
        }

        // Run `zip -r <output_zip> .` inside the resourcepack directory
        let output_zip = out_dir.join(resourcepack_zip_name);
        let status = Command::new("zip")
            .args(&["-r", output_zip.to_str().unwrap(), "."])
            .current_dir(&resourcepack_gen_dir)
            .status()
            .expect("Failed to execute `zip` command");
        if !status.success() {
            panic!("`zip` exited with status: {}", status);
        }

        // Compute SHA1 of the zip and write it to OUT_DIR for include_str!
        let zip_bytes = fs::read(&output_zip).expect("Failed to read output zip");
        let hash_bytes = sha1::Sha1::digest(&zip_bytes);
        let sha1_hash = format!("{:x}", hash_bytes);

        let build_out = PathBuf::from(env::var("OUT_DIR").unwrap());
        let hash_file = build_out.join("resourcepack.sha1");
        fs::write(&hash_file, &sha1_hash).expect("Failed to write SHA1 file");
    }
}