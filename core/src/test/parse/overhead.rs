use rstest::*;
use std::path::PathBuf;

use crate::{
	mul,
	parse::overhead::{parse_file, Weight},
	term::Term,
	val, var,
};

/// Parses hard-coded Storage weight files correctly.
#[rstest]
// Modified from Substrate.
#[case("../test_data/new/extrinsic_weights.rs.txt", Weight::ExtrinsicBase(mul!(val!(85_212), var!("WEIGHT_PER_NANOS"))))]
#[case("../test_data/new/block_weights.rs.txt", Weight::BlockExecution(mul!(val!(5_481_991), var!("WEIGHT_PER_NANOS"))))]
fn parses_weight_files(#[case] file: PathBuf, #[case] want: Weight) {
	let got = parse_file(&file).unwrap();

	assert_eq!(want, got);
}
