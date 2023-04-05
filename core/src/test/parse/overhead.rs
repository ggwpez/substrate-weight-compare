use rstest::*;
use std::path::PathBuf;

use crate::{
	cadd, cmul, cval, cvar,
	parse::overhead::{parse_content, parse_file, Weight},
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

#[rstest]
#[case("parameter_types! {
	/// Time to execute an empty block.
	/// Calculated by multiplying the *Average* with `1.0` and adding `0`.
	///
	/// Stats nanoseconds:
	///   Min, Max: 6_708_387, 7_042_534
	///   Average:  6_818_965
	///   Median:   6_826_464
	///   Std-Dev:  66350.7
	///
	/// Percentiles nanoseconds:
	///   99th: 6_991_352
	///   95th: 6_933_543
	///   75th: 6_854_332
	pub const BlockExecutionWeight: Weight =
		Weight::from_parts(WEIGHT_REF_TIME_PER_NANOS.saturating_mul(6_818_965), 0);
	}", 
	Weight::BlockExecution(cadd!(cmul!(cvar!("WEIGHT_REF_TIME_PER_NANOS"), cval!((6_818_965, 0).into())), cval!((0, 0).into()))))]
fn parses_chromatic_expression(#[case] input: &str, #[case] want: Weight) {
	let got = parse_content(input.into()).unwrap();

	assert_eq!(want, got);
}
