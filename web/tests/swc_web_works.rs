#![cfg(test)]

use assert_cmd::cargo::CommandCargoExt;
use serial_test::serial;
use std::process::Command;

use subweight_core::testing::{assert_version, root_dir, succeeds, KillChildOnDrop};

#[test]
fn subweight_web_version_works() {
	let output = Command::cargo_bin("subweight-web").unwrap().arg("--version").output().unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert_version(&out, "subweight-web");
}

#[test]
fn subweight_web_help_works() {
	let output = Command::cargo_bin("subweight-web").unwrap().arg("--help").output().unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert!(out.contains("Print help"));
}

#[test]
#[serial]
#[cfg_attr(not(feature = "polkadot"), ignore)]
fn subweight_web_url_works() {
	let _cmd = KillChildOnDrop(
		Command::cargo_bin("subweight-web")
			.unwrap()
			.args([
				"--root",
				root_dir().join("repos").to_str().unwrap(),
				"--repos",
				"polkadot",
				"--static",
				"../web/static",
			])
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

		// Search for an example:
		if resp.contains("Polkadot with tags") {
			return
		}
	}
	panic!("Failed to make request in time");
}

#[test]
#[serial]
#[cfg_attr(not(feature = "polkadot"), ignore)]
fn subweight_web_compare_works() {
	let _cmd = KillChildOnDrop(
		Command::cargo_bin("subweight-web")
			.unwrap()
			.args([
				"--root",
				root_dir().join("repos").to_str().unwrap(),
				"--repos",
				"polkadot",
				"--static",
				"../web/static",
			])
			.env("RUST_LOG", "error")
			.spawn()
			.unwrap(),
	);

	for _ in 0..20 {
		std::thread::sleep(std::time::Duration::from_millis(100));

		let url = "http://localhost:8080/compare?old=v0.9.19&new=v0.9.20&repo=polkadot&threshold=10&unit=weight&path_pattern=runtime/polkadot/src/weights/*.rs&method=base&ignore_errors=false&git_pull=false";
		let resp = reqwest::blocking::get(url).expect("Request error").text().unwrap();

		// Some magic numbers: utility::batch_all and staking::validate old equations
		if !resp.contains("12.68M + 4.41M * c + READ + WRITE") ||
			!resp.contains("41.30M + 12 * READ + 8 * WRITE")
		{
			panic!("Unexpected response: {}", resp);
		} else {
			return
		}
	}
	panic!("Failed to make request in time");
}
