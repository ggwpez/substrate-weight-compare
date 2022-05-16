pub mod helper;
mod integration;
mod overhead;
mod pallet;
mod storage;

use rstest::*;
use std::path::{Path, PathBuf};

use crate::{
	compare_files, filter_changes,
	parse::{pallet::parse_files, PathStripping, PathStripping::*},
	testing::assert_contains,
	CompareMethod,
};

#[rstest]
#[case("repo/pallet.rs", ".", FileName, "pallet.rs")]
#[case("repo/pallet.rs", "repo", FileName, "pallet.rs")]
#[case("repo/pallet.rs", ".", RepoRelative, "repo/pallet.rs")]
#[case("repo/pallet.rs", "repo", RepoRelative, "pallet.rs")]
fn path_stripping_works(
	#[case] path: String, #[case] repo: String, #[case] mode: PathStripping, #[case] output: String) {
	let path = Path::new(&path);
	let repo = Path::new(&repo);

	assert_eq!(output, mode.strip(&repo, &path))
}
