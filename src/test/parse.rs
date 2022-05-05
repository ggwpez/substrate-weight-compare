use rstest::*;
use std::path::PathBuf;

use crate::parse::parse_file;

/// Parses hard-coded weight files.
#[rstest]
#[case("test_data/new/pallet_staking.rs.txt")]
#[case("test_data/old/pallet_staking.rs.txt")]
fn parses_weight_files(#[case] path: PathBuf) {
    assert!(parse_file(&path).is_ok());
}
