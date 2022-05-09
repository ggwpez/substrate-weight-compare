pub mod helper;
use crate::integration_test;

integration_test!(
	polkadot,
	"polkadot",
	"568169b41aea59a54ab8cfa23c31e84a26708280",
	804,
	138,
	10,
	vec![
		"runtime/*/src/weights/**/*.rs",
		"bridges/modules/*/src/weights.rs",
		"bridges/primitives/messages/src/source_chain.rs",
		"xcm/xcm-executor/src/traits/drop_assets.rs"
	],
	vec!["runtime/*/constants/src/weights/**/*db_weights.rs"]
);

integration_test!(
	moonbeam,
	"moonbeam",
	"9665bd46a19ef4cc4ad1327f360150d7743dfd76",
	195,
	6,
	0,
	vec!["pallets/*/src/weights.rs", "pallets/parachain-staking/src/traits.rs"],
	vec!["^$"]
);

integration_test!(
	composable,
	"composable",
	"b3492f26dd4fde7aca272bae8460682babbdbdd3",
	344,
	79,
	0,
	vec!["frame/*/src/weights.rs", "runtime/*/src/weights/*.rs"],
	vec!["^$"]
);
