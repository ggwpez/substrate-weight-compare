import { path_pattern_presets } from "./common.js";

// Local storage keys
const lsk_selected_repo = "selected-repo";
const lsk_selected_first = "selected-second";
const lsk_selected_second = "selected-first";

// Maps repos to their branches.
var branches = {};

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

// branches maps the branches to their last commits.
function populate_branches(branches) {
    $('#firstSelect').empty().trigger("change");
    $('#secondSelect').empty().trigger("change");
    $('.branchCount').text(branches.length);

    for (var i = 0; i < branches.length; i++) {
        let branch_name = branches[i][0];
        let last_commit = branches[i][1];
        let text = `${branch_name} on ${last_commit}`;

        var newOption = new Option(text, branch_name, false, false);
        $('#firstSelect').append(newOption);

        // Somehow we need to create this twice, otherwise it does not work.
        var newOption2 = new Option(text, branch_name, false, false);
        $('#secondSelect').append(newOption2);
    }

    // Load the selected branch from local storage or master.
    var selected_first = localStorage.getItem(lsk_selected_first);
    if (selected_first != null && branches.includes(selected_first)) {
        console.debug("Setting first branch to " + selected_first);
        $('#firstSelect').val(selected_first).trigger('change');
    } else {
        console.debug("Setting first branch to master");
        $('#firstSelect').val("master").trigger('change');
    }

    var selected_second = localStorage.getItem(lsk_selected_second);
    if (selected_second != null && branches.includes(selected_second)) {
        console.debug("Setting second branch to " + selected_second);
        $('#secondSelect').val(selected_second).trigger('change');
    } else {
        console.debug("Setting second branch to master");
        $('#secondSelect').val("master").trigger('change');
    }
}

function load_branches(repo, fetch) {
    // Is the repo unknown or should be fetched?
    if (!(repo in branches) || fetch) {
        loading(true);
        console.log("Loading branches for " + repo);
        $.getJSON("/branches?repo=" + repo + "&fetch=" + (fetch ? "true" : "false"), function (d) {
            let data = d['branch'];
            branches[repo] = data;
        }).done(function () {
            populate_branches(branches[repo]);
            loading(false);
        });
    } else {
        populate_branches(branches[repo]);
    }
}

$(document).ready(function () {
    $('.js-example-basic-single').select2();
    $('.js-example-basic-single').val(null).trigger('change');

    var current_fs, next_fs, previous_fs; //fieldsets
    var opacity;

    $(".next").click(function () {

        current_fs = $(this).parent();
        next_fs = $(this).parent().next();

        //Add Class Active
        $("#progressbar li").eq($("fieldset").index(next_fs)).addClass("active");

        //show the next fieldset
        next_fs.show();
        //hide the current fieldset with style
        current_fs.animate({ opacity: 0 }, {
            step: function (now) {
                // for making fielset appear animation
                opacity = 1 - now;

                current_fs.css({
                    'display': 'none',
                    'position': 'relative'
                });
                next_fs.css({ 'opacity': opacity });
            },
            duration: 600
        });
    });

    $(".previous").click(function () {

        current_fs = $(this).parent();
        previous_fs = $(this).parent().prev();

        //Remove class active
        $("#progressbar li").eq($("fieldset").index(current_fs)).removeClass("active");

        //show the previous fieldset
        previous_fs.show();

        //hide the current fieldset with style
        current_fs.animate({ opacity: 0 }, {
            step: function (now) {
                // for making fieldset appear animation
                opacity = 1 - now;

                current_fs.css({
                    'display': 'none',
                    'position': 'relative'
                });
                previous_fs.css({ 'opacity': opacity });
            },
            duration: 600
        });
    });

    $('.radio-group .radio').click(function () {
        $(this).parent().find('.radio').removeClass('selected');
        $(this).addClass('selected');
    });

    $(".submit").click(function () {
        return false;
    })

    $("#bthPost").click(function () {
        const repo = $('#repoSelect').val();

        var url = "/compare?unit=weight&ignore_errors=true&threshold=10&method=asymptotic";
        url += "&repo=" + repo;
        url += "&old=" + $('#firstSelect').val();
        url += "&new=" + $('#secondSelect').val();

        if (repo in path_pattern_presets) {
            url += "&path_pattern=" + path_pattern_presets[repo];
        } else {
            // Show an alert
            alert("No path pattern for '" + repo + "'");
            return;
        }

        console.log(url);
        loading(true);
        window.location.href = url;
    });

    $('#repoSelect').change(function () {
        const repo = $(this).val();

        $('#selectedRepo').val(repo);
        localStorage.setItem(lsk_selected_repo, repo);

        // Load the branches of the repo if not already loaded.
        load_branches(repo, false);
    });
    $('#firstSelect').change(function () {
        if (!$(this).val())
            return;

        $('#selectedFirst').val($(this).val());
        localStorage.setItem(lsk_selected_first, $(this).val());
        console.debug("Storing first branch as " + $(this).val());
    });
    $('#secondSelect').change(function () {
        if (!$(this).val())
            return;
        
        $('#selectedSecond').val($(this).val());
        localStorage.setItem(lsk_selected_second, $(this).val());
        console.debug("Storing second branch as " + $(this).val());
    });

    $('#fetch').click(function () {
        const repo = $('#repoSelect').val();
        if (repo === null)
            return;
        load_branches(repo, true);
    });

    // Load the repo that was last selected
    var selected_repo = localStorage.getItem(lsk_selected_repo);
    if (selected_repo && selected_repo in branches) {
        console.debug("Loading last selected repo: " + selected_repo);
        $('#repoSelect').val(selected_repo).trigger('change');
    } else {
        console.debug("Skipped loading of repo: " + selected_repo);
    }
});
