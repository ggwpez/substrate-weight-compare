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
	"1fd71c7845d6c28c532795ec79106d959dd1fe30",
	1438,
	43,
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
	"4863d0a33a4a3534236f76abb5b1dc91751c6c34",
	653,
	135,
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
	cumulus,
	"cumulus",
	"530a0aaf0c8a422f708415873d1366ae4c8ea456",
	198,
	59,
	12,
	12,
	vec![
		"**/weights/*.rs", "**/src/weights.rs",
	]; exclude=vec![],
	vec!["**/*db_weights.rs"],
	vec!["**/block_weights.rs", "**/extrinsic_weights.rs"]
);

integration_test!(
	acala,
	"acala",
	"c64bb09242bbb8db46ff64a97f30331a3006875e",
	380,
	105,
	0,
	0,
	vec![
		"runtime/*/src/weights/*.rs",
		"modules/*/src/weights.rs",
		// Should be possible to remove in the future.
		"modules/homa-validator-list/src/lib.rs",
	]; exclude=vec![
		"**/mod.rs",
		// This file is just empty, wtf?
		"runtime/common/src/weights/lib.rs",
	],
	vec![],
	vec![]
);

integration_test!(
	moonbeam,
	"moonbeam",
	"54e40e2aa3f1f41a45a7df067a6ac6a0256cda6a",
	234,
	7,
	0,
	0,
	vec!["**/weights.rs", "pallets/parachain-staking/src/traits.rs"]; exclude=vec![],
	vec![],
	vec![]
);

integration_test!(
	astar,
	"astar",
	"94a7b3f87b33f64d66123ee9acc8769c25696aa0",
	47,
	7,
	0,
	0,
	vec!["**/weights.rs", "**/weights/*.rs", "**/weight.rs",
		"frame/ibc/ibc-trait/src/lib.rs"];
	exclude=vec![],
	vec![],
	vec![]
);

integration_test!(
	composable,
	"composable",
	"6f407847041ea170db8ddfb4770e0492e253db1f",
	442,
	87,
	0,
	0,
	vec!["**/weights.rs", "**/weights/*.rs", "**/weight.rs",
		"frame/ibc/ibc-trait/src/lib.rs"];
	exclude=vec![],
	vec![],
	vec![]
);

integration_test!(
	nodle,
	"chain",
	"348a54affc9a6dcd0cc900dee9919c7a0ba98aa8",
	54,
	12,
	0,
	0,
	vec!["pallets/*/src/weights.rs", "runtimes/*/src/weights/*.rs"];
	exclude=vec![],
	vec![],
	vec![]
);
