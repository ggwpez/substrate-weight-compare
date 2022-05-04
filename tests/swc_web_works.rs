use assert_cmd::cargo::CommandCargoExt;
use std::process::Command;

#[test]
fn swc_web_version_works() {
    let output = Command::cargo_bin("swc-web")
        .unwrap()
        .arg("--version")
        .output()
        .unwrap();
    succeeds(&output);

    let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert_eq!(out, "substrate-weight-compare 0.1.1");
}

#[test]
fn swc_web_help_works() {
    let output = Command::cargo_bin("swc-web")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    succeeds(&output);

    let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert!(out.contains("Print help information"));
}

/// Asserts that the command output is successful.
// TODO: Could be done as extension trait.
fn succeeds(output: &std::process::Output) {
    if !output.status.success() {
        panic!("{}", String::from_utf8_lossy(&output.stderr));
    }
}
