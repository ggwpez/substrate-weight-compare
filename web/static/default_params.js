const path_pattern_presets = {
    "substrate": "frame/*/src/weights.rs",
    "polkadot": "runtime/**/src/weights/**/*.rs",
	"cumulus": "parachains/runtimes/**/src/weights/*.rs",
};

export function default_params(repo) {
	const pattern = path_pattern_presets[repo];

	if (pattern == null || repo == null)
		throw new Error("Unknown repository: " + repo);

	return {
		"repo": repo,
		"threshold": "10",
		"path_pattern": pattern,
		"method": "guess-worst",
		"ignore_errors": "true",
		"unit": "weight",
	};
};
