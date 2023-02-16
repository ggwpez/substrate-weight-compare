use rstest::*;
use std::path::PathBuf;

use crate::{
	cmul, cval, cvar,
	parse::overhead::{parse_file, Weight},
	term::*,
};

// Parses hard-coded Storage weight files correctly.
#[rstest]
#[case("../test_data/new/extrinsic_weights.rs.txt", Weight::ExtrinsicBase(cmul!(cval!((85_212, 0).into()), cvar!("WEIGHT_PER_NANOS"))))]
#[case("../test_data/new/block_weights.rs.txt", Weight::BlockExecution(cmul!(cval!((5_481_991, 0).into()), cvar!("WEIGHT_PER_NANOS"))))]
fn parses_weight_files(#[case] file: PathBuf, #[case] want: Weight) {
	let got = parse_file(&file).unwrap();

	assert_eq!(want, got);
}
