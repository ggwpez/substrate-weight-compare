// Expose URL params as vars.
//
// Example: $.urlParam('var_name');
// From: https://stackoverflow.com/a/25359264/10167711
$.param = function(name){
    var results = new RegExp('[\?&]' + name + '=([^&#]*)').exec(window.location.href);
    if (results==null) {
       return null;
    }
    return decodeURI(results[1]) || 0;
}

// Takes query name and value to redirect to.
function url_redirect(arg, value) {
	if (value === null || value === '' || value === undefined) {
		console.warn(`url_redirect: invalid value ${value} for arg ${arg}`);
		return;
	}
	var url = new URL(window.location);
	url.searchParams.set(arg, value);
	console.log("Redirecting to: " + url.toString());

	window.location.href = url.toString();
}

$(document).ready(function () {
	// Setup the data table
	var table = $('#sort_me').DataTable({
		paging: false,
		ordering: true,
		select: {
			items: 'row'
		},
		autoWidth: false,
		responsive: true,
		fixedColumns:   {
            heightMatch: 'none'
        },
	});

	// Select the row from the URL anchor - if any.
	var anchor = $(location).attr('hash').replace('#', '');
	if (anchor) {
		console.info("Selecting row: " + anchor);
		try {
			var row = table.row(`#${anchor}`);
			row.select();
			document.getElementById(anchor).scrollIntoView();
		} catch (_e) {
			// Could be excluded by a filter.
		}
	} else {
		console.info("No anchor found.");
	}
	
	// Init the selectors.
	const selectors = ["unit", "method", "repo"]
	for (const selector of selectors) {
		const id = `#select_${selector}`;
		// Redirect on change.
		$(id).change(function() {
			url_redirect(selector, $(this).val());
		});
	}
	// Init the input boxes.
	const inputs = ["threshold", "path_pattern", "old", "new"];
	for (const input of inputs) {
		const id = `#input_${input}`;
		// Redirect on change.
		$(id).change(function() {
			url_redirect(input, $(this).val());
		});
	}
	// Init the checkboxes.
	const checkboxes = ["ignore_errors"];
	for (const checkbox of checkboxes) {
		const id = `#checkbox_${checkbox}`;
		// Redirect on change.
		$(id).change(function() {
			console.log("Checkbox changed: " + checkbox);
			url_redirect(checkbox, $(this).is(":checked"));
		});
	}
});
