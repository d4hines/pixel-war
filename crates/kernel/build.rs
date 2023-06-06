use std::process::Command;

fn main() {
    // Run `git rev-parse HEAD` to get the current commit hash
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();

    // Convert the output into a string
    let git_hash = String::from_utf8(output.stdout).unwrap();

    // Pass the git hash to the compiler via the GIT_HASH env var
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
