//! Moonbeam integration tests.
//!
//! One for a locked version which must succeed and an optional one for master.

#![cfg(test)]

use crate::integration_test;

integration_test!(
	polkadot,
	"polkadot",
	"568169b41aea59a54ab8cfa23c31e84a26708280",
	804,
	133,
	10,
	"runtime/*/src/weights/**/*.rs",
	"runtime/*/constants/src/weights/**/*db_weights.rs"
);

integration_test!(
	composable,
	"composable",
	"b3492f26dd4fde7aca272bae8460682babbdbdd3",
	344,
	17,
	0,
	"frame/*/src/weights.rs",
	"^$"
);

integration_test!(
	moonbeam,
	"moonbeam",
	"9665bd46a19ef4cc4ad1327f360150d7743dfd76",
	195,
	5,
	0,
	"pallets/*/src/weights.rs",
	"^$"
);
