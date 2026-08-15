use std::{
    env,
    path::PathBuf,
    process::Command,
};

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let output_dir =
        manifest_dir.join("../generated/pglite/migrations");

    let status = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "-p",
            "migration",
            "--bin",
            "migration-export",
            "--",
            output_dir
                .to_str()
                .expect("invalid output path"),
        ])
        .current_dir(&manifest_dir)
        .status()
        .expect("failed to run migration exporter");

    if !status.success() {
        panic!("PGlite migration generation failed");
    }

    println!("cargo:rerun-if-changed=migration/src");
}