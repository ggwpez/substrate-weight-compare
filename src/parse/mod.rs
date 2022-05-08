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

pub enum ParseResult {
	Pallet(pallet::ParsedExtrinsic),
	Db(storage::DbWeights),
}

pub fn read_file(file: &Path) -> Result<String, String> {
	let mut file = std::fs::File::open(file).map_err(|e| format!("{:?}", e))?;
	let mut content = String::new();
	file.read_to_string(&mut content).map_err(|e| format!("{:?}", e))?;
	Ok(content)
}
