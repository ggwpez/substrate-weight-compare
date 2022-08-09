import { default_params } from './default_params.js';

// TODO move this in to a common.js
function loading(yes) {
    console.debug("Loading: " + yes);
    if (yes) {
        $("div.spanner").addClass("show");
        $("div.overlay").addClass("show");
    } else {
        $("div.spanner").removeClass("show");
        $("div.overlay").removeClass("show");
    }
}

function should_highlight(mr) {
	// Does the title or the description contain "weight"?
	var text =
		(new String(mr.title).toLowerCase().includes("weight") ||
	     new String(mr.body).toLowerCase().includes("weight"));

	//var branch =
	//	(mr.head.ref.toLowerCase().includes("weight") ||
	//	 mr.base.ref.toLowerCase().includes("weight"));

	//var user = ["chevdor", "coderobe", "ggwpez"].includes(mr.user.login);

	return text;
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
	var repo = $.param('repo', 'polkadot');

	var highlighted = 0;
	// Request the GitHub API to list all merge requests
	// for the given repository and owner.
	$.getJSON(`https://api.github.com/repos/${owner}/${repo}/pulls?state=open&per_page=30&sort=updated&direction=desc`, function(data) {
	// There is a mock for testing:
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

			if (should_highlight(mr)) {
				$(row).css('color', 'seagreen');
				highlighted++;
			}

			// Create a double-click handler for the row:
			(function (mr) {
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
			})(mr);
		}

		// Set the 'highlighted' variable.
		$('#highlighted').text(highlighted);
		loading(false);
	}).await;
});
