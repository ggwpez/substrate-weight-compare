pub mod helper;
mod integration;
mod pallet;
mod storage;

use rstest::*;
use std::path::{Path, PathBuf};

use crate::{
	compare_files, filter_changes, fmt_changes, parse::pallet::parse_files, CompareMethod,
};

/// Compares hard-coded weight files.
#[rstest]
#[case("../test_data/old/pallet_staking.rs.txt", "../test_data/new/pallet_staking.rs.txt", vec!["+74.26", "+6.01", "-7.21"])]
fn compares_weight_files(#[case] old: PathBuf, #[case] new: PathBuf, #[case] expected: Vec<&str>) {
	let old = parse_files(Path::new("."), &[old]).unwrap();
	let new = parse_files(Path::new("."), &[new]).unwrap();

	let diff = compare_files(old, new, 10.0, CompareMethod::Worst);
	let interesting_diff = filter_changes(diff, 5.0);
	let got = fmt_changes(&interesting_diff);

	for exp in expected {
		assert!(got.contains(exp));
	}
}
