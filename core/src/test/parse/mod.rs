pub mod helper;
mod integration;
mod overhead;
mod pallet;
mod storage;

use rstest::*;
use std::path::Path;

use crate::parse::{PathStripping, PathStripping::*};

#[rstest]
#[case("repo/pallet.rs", ".", FileName, "pallet.rs")]
#[case("repo/pallet.rs", "repo", FileName, "pallet.rs")]
#[case("repo/pallet.rs", ".", RepoRelative, "repo/pallet.rs")]
#[case("repo/pallet.rs", "repo", RepoRelative, "pallet.rs")]
fn path_stripping_works(
	#[case] path: String,
	#[case] repo: String,
	#[case] mode: PathStripping,
	#[case] output: String,
) {
	let path = Path::new(&path);
	let repo = Path::new(&repo);

	assert_eq!(output, mode.strip(repo, path))
}

#[test]
fn fancy_regex_works() {
	let regex =
		fancy_regex::Regex::new(r"^(runtimes|pallets)/.*/src/weights(/.*rs|.*rs)$").unwrap();
	let input_pallet = vec![
		"pallets/allocations/src/weights.rs",
		"pallets/grants/src/weights.rs",
		"pallets/reserve/src/weights.rs",
		"pallets/staking/src/weights.rs",
	];
	for input in input_pallet {
		assert!(regex.is_match(input).unwrap());
	}
	let input_runtime = vec![
		"runtimes/eden/src/weights/frame_system.rs",
		"runtimes/eden/src/weights/pallet_multisig.rs",
	];
	for input in input_runtime {
		assert!(regex.is_match(input).unwrap());
	}
}
