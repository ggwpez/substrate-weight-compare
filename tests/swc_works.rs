use assert_cmd::cargo::CommandCargoExt;
use serial_test::serial;
use std::{path::Path, process::Command};

mod common;

use common::succeeds;

const ROOT_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn swc_version_works() {
	let output = Command::cargo_bin("swc").unwrap().arg("--version").output().unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	common::valid_version(&out);
}

#[test]
fn swc_help_works() {
	let output = Command::cargo_bin("swc").unwrap().arg("--help").output().unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert!(out.contains("Print help information"));
}

#[test]
#[serial]
#[cfg_attr(not(feature = "polkadot-tests"), ignore)]
fn swc_compare_commits_works() {
	let output = Command::cargo_bin("swc")
		.unwrap()
		.args(["compare", "commits"])
		.args(["v0.9.19", "v0.9.20"])
		.output()
		.unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert!(out.contains("pallet_election_provider_multi_phase.rs"));
}

#[test]
#[serial]
#[cfg_attr(not(feature = "polkadot-tests"), ignore)]
fn swc_compare_commits_same_no_changes() {
	let output = Command::cargo_bin("swc")
		.unwrap()
		.args(["compare", "commits"])
		.args(["v0.9.19", "v0.9.19"])
		.output()
		.unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert!(out.contains("No changes found."));
}

#[test]
#[serial]
#[cfg_attr(not(feature = "polkadot-tests"), ignore)]
fn swc_compare_commits_errors() {
	let output = Command::cargo_bin("swc")
		.unwrap()
		.args(["compare", "commits"])
		.args(["vWrong"])
		.output()
		.unwrap();
	assert!(!output.status.success());

	let out = String::from_utf8_lossy(&output.stderr).trim().to_owned();
	assert!(out.contains("revspec 'vWrong' not found"));
}

#[test]
fn swc_compare_files_works() {
	let output = Command::cargo_bin("swc")
		.unwrap()
		.args(["compare", "files"])
		.args([
			"--old",
			Path::new(ROOT_DIR)
				.join("test_data/old/pallet_staking.rs.txt")
				.to_str()
				.unwrap(),
			"--new",
			Path::new(ROOT_DIR)
				.join("test_data/new/pallet_staking.rs.txt")
				.to_str()
				.unwrap(),
			"--threshold",
			"0",
		])
		.output()
		.unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert!(out.contains("payout_stakers_dead_controller"));
}

#[test]
fn swc_compare_files_same_no_changes() {
	let output = Command::cargo_bin("swc")
		.unwrap()
		.args(["compare", "files"])
		.args([
			"--old",
			Path::new(ROOT_DIR)
				.join("test_data/new/pallet_staking.rs.txt")
				.to_str()
				.unwrap(),
			"--new",
			Path::new(ROOT_DIR)
				.join("test_data/new/pallet_staking.rs.txt")
				.to_str()
				.unwrap(),
			"--threshold",
			"0",
		])
		.output()
		.unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert!(out.contains("No changes found."));
}

#[test]
fn swc_compare_files_errors() {
	let output = Command::cargo_bin("swc")
		.unwrap()
		.args(["compare", "files"])
		.args([
			"--old",
			Path::new(ROOT_DIR)
				.join("src/lib.rs") // Pass in a wrong file.
				.to_str()
				.unwrap(),
			"--new",
			Path::new(ROOT_DIR)
				.join("test_data/new/pallet_staking.rs.txt")
				.to_str()
				.unwrap(),
			"--threshold",
			"0",
		])
		.output()
		.unwrap();
	assert!(!output.status.success());

	let out = String::from_utf8_lossy(&output.stderr).trim().to_owned();
	assert!(out.contains("Could not find weight implementation in the passed file"));
}
