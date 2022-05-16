#![cfg(test)]

use assert_cmd::cargo::CommandCargoExt;
use serial_test::serial;
use std::process::Command;

use swc_core::testing::{assert_contains, assert_version, root_dir, succeeds, KillChildOnDrop};

#[test]
fn swc_web_version_works() {
	let output = Command::cargo_bin("swc-web").unwrap().arg("--version").output().unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert_version(&out, "swc_web");
}

#[test]
fn swc_web_help_works() {
	let output = Command::cargo_bin("swc-web").unwrap().arg("--help").output().unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert!(out.contains("Print help information"));
}

#[test]
#[serial]
#[cfg_attr(not(feature = "polkadot"), ignore)]
fn swc_web_url_works() {
	let _cmd = KillChildOnDrop(
		Command::cargo_bin("swc-web")
			.unwrap()
			.args(["--repo", root_dir().join("repos/polkadot").to_str().unwrap()])
			.env("RUST_LOG", "error")
			.spawn()
			.unwrap(),
	);

	for _ in 0..20 {
		std::thread::sleep(std::time::Duration::from_millis(100));

		let resp = reqwest::blocking::get("http://localhost:8080/compare")
			.expect("Request error")
			.text()
			.unwrap();

		if resp.contains("Example #1") {
			return
		}
	}
	panic!("Failed to make request in time");
}

#[test]
#[serial]
#[cfg_attr(not(feature = "polkadot"), ignore)]
fn swc_web_compare_works() {
	let _cmd = KillChildOnDrop(
		Command::cargo_bin("swc-web")
			.unwrap()
			.args(["--repo", root_dir().join("repos/polkadot").to_str().unwrap()])
			.env("RUST_LOG", "error")
			.spawn()
			.unwrap(),
	);

	for _ in 0..20 {
		std::thread::sleep(std::time::Duration::from_millis(100));

		let url = "http://localhost:8080/compare?old=v0.9.19&new=v0.9.20&threshold=10&path_pattern=runtime/polkadot/src/weights/*.rs&method=base&ignore_errors=false";
		let resp = reqwest::blocking::get(url).expect("Request error").text().unwrap();

		// Some magic numbers
		if !resp.contains("-99.46") || !resp.contains("33.00K") {
			panic!("Unexpected response: {}", resp);
		} else {
			return;
		}
	}
	panic!("Failed to make request in time");
}
