pub mod helper;
mod pallet;
mod storage;

use rstest::*;
use std::path::PathBuf;

use crate::{fmt_changes, extract_changes,compare_files};
use crate::parse::pallet::parse_files;

/// Compares hard-coded weight files.
#[rstest]
#[case("../test_data/old/pallet_staking.rs.txt", "../test_data/new/pallet_staking.rs.txt", vec!["+74.26", "+6.01", "-7.21"])]
fn compares_weight_files(#[case] old: PathBuf, #[case] new: PathBuf, #[case] expected: Vec<&str>) {
	let old = parse_files(&vec![old]).unwrap();
	let new = parse_files(&vec![new]).unwrap();

	let diff = compare_files(old, new);
	let interesting_diff = extract_changes(diff, 5.0);
	let got = fmt_changes(&interesting_diff);

	for exp in expected {
		assert!(got.contains(exp));
	}
}
