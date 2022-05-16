use assert_cmd::cargo::CommandCargoExt;
use serial_test::serial;
use std::process::Command;

use swc_core::testing::{assert_contains, assert_not_contains, assert_version, root_dir, succeeds};

#[test]
fn swc_version_works() {
	let output = Command::cargo_bin("swc").unwrap().arg("--version").output().unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert_version(&out, "swc");
}

#[test]
fn swc_help_works() {
	let output = Command::cargo_bin("swc").unwrap().arg("--help").output().unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert_contains(&out, "Print help information");
}

#[test]
#[serial]
#[cfg_attr(not(feature = "polkadot"), ignore)]
fn swc_compare_commits_works() {
	let output = Command::cargo_bin("swc")
		.unwrap()
		.args([
			"compare",
			"commits",
			"--method",
			"base",
			"--path-pattern",
			"runtime/polkadot/src/weights/*.rs",
		])
		.args(["v0.9.19", "v0.9.20"])
		.args(["--repo", root_dir().join("repos/polkadot").to_str().unwrap()])
		.output()
		.unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert_contains(&out, "pallet_election_provider_multi_phase.rs");
}

#[test]
#[serial]
#[cfg_attr(not(feature = "polkadot"), ignore)]
fn swc_compare_commits_same_no_changes() {
	let output = Command::cargo_bin("swc")
		.unwrap()
		.args([
			"compare",
			"commits",
			"--method",
			"base",
			"--path-pattern",
			"runtime/polkadot/src/weights/*.rs",
		])
		.args(["v0.9.19", "v0.9.19"])
		.args(["--repo", root_dir().join("repos/polkadot").to_str().unwrap()])
		.output()
		.unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert_contains(&out, "No changes found.");
}

#[test]
#[serial]
#[cfg_attr(not(feature = "polkadot"), ignore)]
fn swc_compare_commits_errors() {
	let output = Command::cargo_bin("swc")
		.unwrap()
		.args(["compare", "commits", "--method", "base", "--path-pattern", "**/*.rs"])
		.args(["vWrong"])
		.args(["--repo", root_dir().join("repos/polkadot").to_str().unwrap()])
		.output()
		.unwrap();
	assert!(!output.status.success());

	let out = String::from_utf8_lossy(&output.stderr).trim().to_owned();
	assert_contains(&out, "revspec 'vWrong' not found");
}

#[test]
fn swc_compare_files_works() {
	let output = Command::cargo_bin("swc")
		.unwrap()
		.args(["compare", "files", "--method", "base"])
		.args([
			"--old",
			root_dir().join("test_data/old/pallet_staking.rs.txt").to_str().unwrap(),
			"--new",
			root_dir().join("test_data/new/pallet_staking.rs.txt").to_str().unwrap(),
			"--threshold",
			"0",
		])
		.output()
		.unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert_not_contains(&out, "Removed");
	assert_not_contains(&out, "Added");

}

#[test]
fn swc_compare_files_same_no_changes() {
	let output = Command::cargo_bin("swc")
		.unwrap()
		.args(["compare", "files", "--method", "base"])
		.args([
			"--old",
			root_dir().join("test_data/new/pallet_staking.rs.txt").to_str().unwrap(),
			"--new",
			root_dir().join("test_data/new/pallet_staking.rs.txt").to_str().unwrap(),
			"--threshold",
			"0",
		])
		.output()
		.unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	let newlines = out.matches("No change").count();
	assert_eq!(newlines, 30); // There are 30 extrinsics in the file.
}

#[test]
fn swc_compare_files_errors() {
	let output = Command::cargo_bin("swc")
		.unwrap()
		.args(["compare", "files", "--method", "base"])
		.args([
			"--old",
			root_dir()
				.join("cli/src/main.rs") // Pass in a wrong file.
				.to_str()
				.unwrap(),
			"--new",
			root_dir().join("test_data/new/pallet_staking.rs.txt").to_str().unwrap(),
			"--threshold",
			"0",
		])
		.output()
		.unwrap();
	assert!(!output.status.success());

	let out = String::from_utf8_lossy(&output.stderr).trim().to_owned();
	assert_contains(&out, "Could not find a weight implementation in the passed file");
}
