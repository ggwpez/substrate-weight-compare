export function loading(yes) {
    console.debug("Loading: " + yes);
    if (yes) {
        $("div.spanner").addClass("show");
        $("div.overlay").addClass("show");
    } else {
        $("div.spanner").removeClass("show");
        $("div.overlay").removeClass("show");
    }
}

export const path_pattern_presets = {
	"substrate": "frame/**/src/weights.rs",
	"polkadot": "runtime/*/src/weights/**/*.rs,bridges/modules/*/src/weights.rs",
	"cumulus": "**/weights/*.rs,**/weights/xcm/*.rs,**/src/weights.rs",
	"chain": "pallets/*/src/weights.rs,runtimes/*/src/weights/*.rs",
};

export function default_params(repo) {
	const pattern = path_pattern_presets[repo];

	if (pattern == null || repo == null)
		throw new Error("Unknown repository: " + repo);

	return {
		"repo": repo,
		"threshold": "10",
		"path_pattern": pattern,
		"method": "asymptotic",
		"ignore_errors": "true",
		"unit": "time",
	};
};
