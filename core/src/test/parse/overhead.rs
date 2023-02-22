use rstest::*;
use std::path::PathBuf;

use crate::{
	cmul, cval, cvar,
	parse::overhead::{parse_file, Weight},
};

// Parses hard-coded Storage weight files correctly.
#[rstest]
#[case("../test_data/new/extrinsic_weights.rs.txt", Weight::ExtrinsicBase(cmul!(cval!((85_212, 0).into()), cvar!("WEIGHT_PER_NANOS"))))]
#[case("../test_data/new/block_weights.rs.txt", Weight::BlockExecution(cmul!(cval!((5_481_991, 0).into()), cvar!("WEIGHT_PER_NANOS"))))]
fn parses_weight_files(#[case] file: PathBuf, #[case] want: Weight) {
	let got = parse_file(&file).unwrap();

	assert_eq!(want, got);
}

/// Parses hard-coded Storage weight files correctly.
#[rstest]
#[case("../test_data/chromatic/extrinsic_weights.rs.txt", Weight::ExtrinsicBase(cmul!(cvar!("WEIGHT_REF_TIME_PER_NANOS"), cval!((99_840, 0).into()))))]
#[case("../test_data/chromatic/block_weights.rs.txt", Weight::BlockExecution(cmul!(cvar!("WEIGHT_REF_TIME_PER_NANOS"), cval!((381_015, 0).into()))))]
fn parses_chromatic_weight_files(#[case] file: PathBuf, #[case] want: Weight) {
	let got = parse_file(&file).unwrap();

	assert_eq!(want, got);
}
