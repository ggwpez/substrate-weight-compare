use rstest::*;
use std::path::PathBuf;

use crate::{
	parse::storage::{parse_file, Db, DbWeights, RWs},
	term::Term,
};

/// Parses hard-coded DB weight files correctly.
#[rstest]
// Modified from Substrate.
#[case("test_data/new/rocksdb_weights.rs.txt", 25, 100, Db::Rocks, true)]
#[case("test_data/new/paritydb_weights.rs.txt", 8, 50, Db::Parity, false)]
fn parses_weight_files(
	#[case] file: PathBuf,
	#[case] read: u128,
	#[case] write: u128,
	#[case] db: Db,
	#[case] per_nanos: bool,
) {
	let want = make_weights(read, write, db, per_nanos);
	let got = parse_file(&file).unwrap();

	assert_eq!(want, got);
}

fn make_weights(read: u128, write: u128, db: Db, per_nanos: bool) -> DbWeights {
	let mut read = Term::Value(read);
	let mut write = Term::Value(write);

	if per_nanos {
		read = Term::Mul(read.into(), Term::Var("constants::WEIGHT_PER_NANOS".into()).into());
		write = Term::Mul(write.into(), Term::Var("constants::WEIGHT_PER_NANOS".into()).into());
	}
	DbWeights { weights: RWs { read, write }, db }
}
