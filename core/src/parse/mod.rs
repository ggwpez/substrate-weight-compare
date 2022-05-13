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

/// Tries to guess the type of weight file and parses it.
///
/// Does not return an error since it just *tires* to do so, not guarantee.
pub fn try_parse_file(repo: &Path, file: &Path) -> Option<ParsedFile> {
	if let Ok(parsed) = pallet::parse_file(repo, file) {
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
	let mut raw = std::fs::File::open(file).map_err(|e| format!("{}: {:?}", file.display(), e))?;
	let mut content = String::new();
	raw.read_to_string(&mut content)
		.map_err(|e| format!("{}: {:?}", file.display(), e))?;
	Ok(content)
}
