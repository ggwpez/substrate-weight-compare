#![cfg(test)]

use glob::glob;
use rstest::*;
use serial_test::serial;
use std::path::PathBuf;

use swc::{checkout, parse::pallet::parse_file};

/// These tests only work on Polkadot master and are therefore not run by default.
/// They must possibly be updated on every Polkadot update.
mod version_locked {
	use super::*;

	/// The number of rust files in the Polkadot repo.
	///
	/// Can be verified with
	/// `ls repos/polkadot/**/*.rs | wc -l`.
	/// NOTE: This needs to be updated on every Polkadot update.
	const NUM_RUST_FILES: usize = 812;

	/// The number of extrinsic weight files in the Polkadot repo.
	///
	/// Can be verified manually by running
	/// `ls repos/polkadot/runtime/*/src/weights/**/*.rs | wc -l`
	/// and filtering out the `mod.rs` files.
	/// NOTE: This needs to be updated on every Polkadot update.
	const NUM_EXT_WEIGHT_FILES: usize = 133;

	/// The number of database weight files in the Polkadot repo.
	///
	/// Can be verified manually by running
	/// `ls repos/polkadot/runtime/*/constants/src/weights/**/*db_weights.rs | wc -l`
	const NUM_DB_WEIGHT_FILES: usize = 10;

	/// Ensure that Polkadot master is checked out.
	///
	/// Other tests could have messed it up.
	fn init() {
		if let Err(err) = checkout(&polkadot_root(), "190515004445a60a54711547765baf7e5bcb5e6d") {
			panic!("Folder `repos/polkadot` must contain the Polkadot repo: {}", err);
		}
	}

	/// Asserts that the correct number of rust files is found.
	#[test]
	#[serial]
	#[cfg_attr(not(all(feature = "polkadot-tests", feature = "version-locked-tests")), ignore)]
	fn num_rust_files() {
		init();
		assert_eq!(polkadot_rust_files().len(), NUM_RUST_FILES);
	}

	/// Asserts that the correct number of weight files is found.
	#[test]
	#[serial]
	#[cfg_attr(not(all(feature = "polkadot-tests", feature = "version-locked-tests")), ignore)]
	fn num_ext_weight_files() {
		init();
		assert_eq!(polkadot_extrinsic_files().len(), NUM_EXT_WEIGHT_FILES);
	}

	/// Asserts that the correct number of weight files is found.
	#[test]
	#[serial]
	#[cfg_attr(not(all(feature = "polkadot-tests", feature = "version-locked-tests")), ignore)]
	fn num_db_weight_files() {
		init();
		assert_eq!(polkadot_db_files().len(), NUM_DB_WEIGHT_FILES);
	}
}

/// Parses all weight files successfully.
#[rstest]
#[serial]
#[cfg_attr(not(feature = "polkadot-tests"), ignore)]
fn parses_ext_weight_files(polkadot_extrinsic_files: Vec<PathBuf>) {
	for file in polkadot_extrinsic_files {
		parse_file(&file).unwrap();
	}
}

/// Tries to parse all rust files and asserts that the number of successful parses is equal to
/// the number of weight files.
// TODO: Check for equality instead of just length.
#[rstest]
#[serial]
#[cfg_attr(not(feature = "polkadot-tests"), ignore)]
fn parses_exactly_ext_weight_files(
	polkadot_rust_files: Vec<PathBuf>,
	polkadot_extrinsic_files: Vec<PathBuf>,
) {
	let weights = polkadot_rust_files.iter().map(|p| parse_file(p)).filter_map(|r| r.ok());

	assert_eq!(weights.count(), polkadot_extrinsic_files.len());
}

#[rstest]
#[serial]
#[cfg_attr(not(feature = "polkadot-tests"), ignore)]
fn parses_db_weight_files(polkadot_db_files: Vec<PathBuf>) {
	for file in polkadot_db_files {
		swc::parse::storage::parse_file(&file).unwrap();
	}
}

#[rstest]
#[serial]
#[cfg_attr(not(feature = "polkadot-tests"), ignore)]
fn parses_exactly_db_weight_files(
	polkadot_rust_files: Vec<PathBuf>,
	polkadot_db_files: Vec<PathBuf>,
) {
	let weights = polkadot_rust_files
		.iter()
		.map(|p| swc::parse::storage::parse_file(p))
		.filter_map(|r| r.ok());

	assert_eq!(weights.count(), polkadot_db_files.len());
}

// Setup code

/// Returns all weight files from a polkadot repository.
#[fixture]
fn polkadot_extrinsic_files() -> Vec<PathBuf> {
	let root = format!("{}/runtime/*/src/weights/**/*.rs", polkadot_root().to_string_lossy());
	glob(&root)
		.unwrap()
		.map(|f| f.unwrap())
		.filter(|f| !f.ends_with("mod.rs"))
		.collect()
}

/// Returns all weight files from a polkadot repository.
#[fixture]
fn polkadot_db_files() -> Vec<PathBuf> {
	let root = format!(
		"{}/runtime/*/constants/src/weights/**/*db_weights.rs",
		polkadot_root().to_string_lossy()
	);
	glob(&root).unwrap().map(|f| f.unwrap()).collect()
}

/// Returns the number of rust files in the Polkadot repository.
#[fixture]
fn polkadot_rust_files() -> Vec<PathBuf> {
	let root = format!("{}/**/*.rs", polkadot_root().to_string_lossy());
	glob(&root).unwrap().map(|f| f.unwrap()).collect()
}

/// Returns the root directory to the Polkadot git repository.
fn polkadot_root() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("repos/polkadot")
}
