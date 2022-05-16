use crate::integration_test;

// Maintenance note for adding a test:
// - Add a new macro instantiation here
// - Clone the project to `repos/`
// - Check out the master commit and lock it here
// - Fix all other params…
// - Add a feature to the Cargo.toml and try it!

integration_test!(
	substrate,
	"substrate",
	"4fd77a94e1aa516c7eb9f6a0428f81637fe87f07",
	1399,
	41,
	2,
	2,
	vec!["**/weights.rs"]; exclude=vec!["frame/support/src/weights.rs"],
	// Keep the patterns in the most general way to catch new files.
	vec!["**/*db_weights.rs"],
	vec!["**/block_weights.rs", "**/extrinsic_weights.rs"]
);

integration_test!(
	polkadot,
	"polkadot",
	"568169b41aea59a54ab8cfa23c31e84a26708280",
	804,
	138,
	10,
	10,
	vec![
		"runtime/*/src/weights/**/*.rs",
		"bridges/modules/*/src/weights.rs",
		"bridges/primitives/messages/src/source_chain.rs",
		"xcm/xcm-executor/src/traits/drop_assets.rs"
	]; exclude=vec![],
	// Keep the patterns in the most general way to catch new files.
	vec!["**/*db_weights.rs"],
	vec!["**/block_weights.rs", "**/extrinsic_weights.rs"]
);

integration_test!(
	moonbeam,
	"moonbeam",
	"9665bd46a19ef4cc4ad1327f360150d7743dfd76",
	195,
	6,
	0,
	0,
	vec!["**/weights.rs", "pallets/parachain-staking/src/traits.rs"]; exclude=vec![],
	vec![],
	vec![]
);

integration_test!(
	composable,
	"composable",
	"b3492f26dd4fde7aca272bae8460682babbdbdd3",
	344,
	79,
	0,
	0,
	vec!["**/weights.rs", "**/weights/*.rs"]; exclude=vec![],
	vec![],
	vec![]
);
