//! Parses weight files of Substrate chains.
//!
//! The following categories are supported:
//! - Extrinsic weights (often pallet_name.rs)
//! - Database weights (often rocksdb_weights.rs or paritydb_weights.rs)
//! - Extrinsic Base weight (often extrinsic_weight.rs)
//! - Block Execution weight (often block_weight.rs)
//!
//! Each module corresponds to one of these categories.

pub mod database;
pub mod extrinsic;
pub use extrinsic::*;
