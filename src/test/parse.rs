#![cfg(test)]

use glob::glob;
use rstest::*;
use std::path::PathBuf;

use crate::parse::parse_file;

/// These tests only work on Polkadot master and are therefore not run by default.
/// They must possibly be updated on every Polkadot update.
mod version_locked {
    use super::*;

    /// The number of rust files in the Polkadot repo.
    ///
    /// Can be verified with
    /// `ls test_data/polkadot_new/**/*.rs | wc -l`.
    /// NOTE: This needs to be updated on every Polkadot update.
    const NUM_RUST_FILES: usize = 803;

    /// The number of weight files in the Polkadot repo.
    ///
    /// Can be verified manually by running
    /// `ls test_data/polkadot_new/runtime/*/src/weights/**/*.rs | wc -l`
    /// and filtering out the `mod.rs` files.
    /// NOTE: This needs to be updated on every Polkadot update.
    const NUM_WEIGHT_FILES: usize = 132;

    /// Asserts that the correct number of rust files is found.
    #[test]
    #[cfg_attr(not(feature = "version_locked_tests"), ignore)]
    fn num_rust_files() {
        assert_eq!(polkadot_rust_files().len(), NUM_RUST_FILES);
    }

    /// Asserts that the correct number of weight files is found.
    #[test]
    #[cfg_attr(not(feature = "version_locked_tests"), ignore)]
    fn num_weight_files() {
        assert_eq!(polkadot_weight_files().len(), NUM_WEIGHT_FILES);
    }
}

/// Parses all weight files successfully.
#[rstest]
fn parses_all_weight_files(polkadot_weight_files: Vec<PathBuf>) {
    for file in polkadot_weight_files {
        parse_file(&file).unwrap();
    }
}

/// Tries to parse all rust files and asserts that the number of successful parses is equal to
/// the number of weight files.
// TODO: Check for equality instead of just length.
#[rstest]
fn parse_extracts_weight_files(
    polkadot_rust_files: Vec<PathBuf>,
    polkadot_weight_files: Vec<PathBuf>,
) {
    let weights = polkadot_rust_files
        .iter()
        .map(parse_file)
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    assert_eq!(weights.len(), polkadot_weight_files.len());
}

/// Returns all weight files from a polkadot repository.
#[fixture]
fn polkadot_weight_files() -> Vec<PathBuf> {
    let root = format!(
        "{}/runtime/*/src/weights/**/*.rs",
        polkadot_root().to_string_lossy()
    );
    glob(&root)
        .unwrap()
        .map(|f| f.unwrap())
        .filter(|f| !f.ends_with("mod.rs"))
        .collect()
}

/// Returns the number of rust files in the Polkadot repository.
#[fixture]
fn polkadot_rust_files() -> Vec<PathBuf> {
    let root = format!("{}/**/*.rs", polkadot_root().to_string_lossy());
    glob(&root).unwrap().map(|f| f.unwrap()).collect()
}

/// Returns the root directory to the Polkadot git repository.
fn polkadot_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data/polkadot_new")
}
