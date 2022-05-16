//! Parses weight files of Substrate chains.
//!
//! The following categories are supported:
//! - Extrinsic weights (often pallet_name.rs)
//! - Database weights (often rocksdb_weights.rs or paritydb_weights.rs)
//! - Extrinsic Base weight (often extrinsic_weight.rs)
//! - Block Execution weight (often block_weight.rs)
//!
//! Each module corresponds to one of these categories.

pub mod overhead;
pub mod pallet;
pub mod storage;

use std::{io::Read, path::Path};

pub enum ParsedFile {
	Pallet(Vec<pallet::Extrinsic>),
	Storage(storage::Weights),
	Overhead(overhead::Weight),
}

/// Defines how a path is transformed into a pallet name.
///
/// Take the following example:
///
///  Compare `old/pallet.rs` to `new/pallet.rs`.
///  Obviously the pallet should have the same name in both cases.
///  The solution is to only look at the file name.
///
/// Next example:
///
///  Compare `polkadot/pallet.rs` to `polkadot/template/pallet.rs`.
///  It this case the pallet name is different, since it are two distinct runtimes.
///  The pallets should not be compared to each other but registered as `Added`.
#[derive(Copy, clap::ArgEnum, PartialEq, Clone, Debug)]
pub enum PathStripping {
	/// Only the file name.
	FileName,
	/// The path relative to the repository.
	RepoRelative,
}

impl PathStripping {
	pub fn strip(&self, repo: &Path, path: &Path) -> String {
		match self {
			Self::FileName => path.file_name().unwrap().to_string_lossy(),
			Self::RepoRelative => path.strip_prefix(repo).unwrap_or(path).to_string_lossy(),
		}
		.into_owned()
	}

	pub fn variants() -> Vec<&'static str> {
		vec!["file_name", "repo_relative"]
	}
}

impl std::str::FromStr for PathStripping {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, String> {
		match s {
			"file_name" => Ok(Self::FileName),
			"repo_relative" => Ok(Self::RepoRelative),
			_ => Err(format!("Unknown stripping: {}", s)),
		}
	}
}

/// Tries to guess the type of weight file and parses it.
///
/// Does not return an error since it just *tires* to do so, not guarantee.
pub fn try_parse_file(repo: &Path, file: &Path) -> Option<ParsedFile> {
	if let Ok(parsed) = pallet::parse_file_in_repo(repo, file) {
		return Some(ParsedFile::Pallet(parsed))
	}
	if let Ok(parsed) = storage::parse_file(file) {
		return Some(ParsedFile::Storage(parsed))
	}
	if let Ok(parsed) = overhead::parse_file(file) {
		return Some(ParsedFile::Overhead(parsed))
	}

	None
}

pub fn read_file(file: &Path) -> Result<String, String> {
	let mut raw = std::fs::File::options()
		.read(true)
		.write(false)
		.open(file)
		.map_err(|e| format!("{}: {:?}", file.display(), e))?;
	let mut content = String::new();
	raw.read_to_string(&mut content)
		.map_err(|e| format!("{}: {:?}", file.display(), e))?;
	Ok(content)
}

pub(crate) fn path_to_string(p: &syn::Path, delimiter: Option<&str>) -> String {
	p.segments
		.iter()
		.map(|s| s.ident.to_string())
		.collect::<Vec<_>>()
		.join(delimiter.unwrap_or_default())
}
