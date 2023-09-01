import { default_params, loading } from './common.js';

function should_highlight(mr) {
	// Does the title contain "weight"?
	var text = new String(mr.title).toLowerCase().includes("weight");

	var branch =
		(mr.head.ref.toLowerCase().includes("weight") ||
		 mr.base.ref.toLowerCase().includes("weight"));

	return text || branch;
}

// Expose URL params as vars.
//
// Example: $.param('var_name');
// From: https://stackoverflow.com/a/25359264/10167711
$.param = function(name, def){
    var results = new RegExp('[\?&]' + name + '=([^&#]*)').exec(window.location.href);
    if (results === null) {
       return def;
    }
    const res = decodeURI(results[1]);
	if (res === null || res === '' || res === undefined) {
		return def;
	}
	return res;
}

function init_ui() {
	// Redirect on change.
	$(`#select_repo`).change(function() {
		// Split and get both owner:repo
		const splits = $(this).val().split(":");
		var url = new URL(window.location);
		url.searchParams.set("owner", splits[0]);
		url.searchParams.set("repo", splits[1]);
		console.log("Redirecting to: " + url.toString());
		window.location.href = url.toString();
	});
}

$(document).ready(function () {
	loading(true);
	var table = $('#mrTable').DataTable({
		paging: false,
		//ordering: true,
		select: {
			items: 'row'
		},
		autoWidth: false,
		responsive: true,
		fixedColumns:   {
            heightMatch: 'none'
        },
		aaSorting: [[ 4, "desc" ]]
	});

	var owner = $.param('owner', 'paritytech');
	var repo = $.param('repo', 'polkadot-sdk');
	$('#select_repo').val(owner + ':' + repo);
	init_ui();

	var highlighted = 0;
	// Request the GitHub API to list all merge requests
	// for the given repository and owner.
	$.getJSON(`https://api.github.com/repos/${owner}/${repo}/pulls?state=open&per_page=100&sort=updated&direction=desc`, function(data) {
	// Use this pre-downloaded data for testing:
	//$.getJSON(`static/dummy-mrs.json`, function(data) {
		// Sort the data by the last push date. This is kind of bad but the table
		// somehow ignores the data-sort attribute if I add via JS...
		data.sort(function(a, b) {
			return new Date(b.updated_at) - new Date(a.updated_at);
		});

		for (var i = 0; i < data.length; i++) {
			var mr = data[i];
			var last_push = new Date(mr.updated_at);
			var creator = mr.user.login;
			var row = table.row.add([
				mr.title.substr(0, 100),
				creator,
				mr['head']['ref'],
				mr['base']['ref'],
				last_push.toLocaleDateString(),
			]).draw().node();

			// Create a double-click handler for the row:
			let click_compare = (function (mr) {
				$(row).dblclick(function() {
					loading(true);
					let params = new URLSearchParams(default_params(repo));
					let url = "/compare?" + params.toString() + `&old=${mr.base.ref}&new=${mr.head.ref}`;
					console.log("Opening: " + url);
					$.getJSON("/branches?repo=" + repo + "&fetch=true", function (d) {
						loading(false);
						window.location.href = url;
					}).done(function () {
						populate_branches(branches[repo]);
						loading(false);
					});					
				});
				/*$(row).click(function() {
					window.location.href = mr.html_url;
				});*/
			});
			let disable = (function (mr) {
				// Disable the row.
				$(row).css('color', 'gray');
				$(row).attr('title', 'Forks are unsupported. Remote: ' + mr['head']['repo']['owner']['login'] + ' != ' + owner);
			});

			if (mr['head']['repo']['owner']['login'] != owner) {
				$(row).css('color', 'gray');
				disable(mr);
			} else {
				if (should_highlight(mr)) {
					$(row).css('color', 'seagreen');
					highlighted++;
				}
				click_compare(mr);
			}
		}

		// Set the 'highlighted' variable.
		$('#highlighted').text(highlighted);
		loading(false);
	}).await;
});
