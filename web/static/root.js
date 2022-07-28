const path_pattern_presets = {
    "substrate": "frame/*/src/weights.rs",
    "polkadot": "runtime/**/src/weights/**/*.rs",
};

var branches = {};

function loading(yes) {
    if (yes) {
        $("div.spanner").addClass("show");
        $("div.overlay").addClass("show");
    } else {
        $("div.spanner").removeClass("show");
        $("div.overlay").removeClass("show");
    }
}

function populate_branches(branches) {
    $('#firstSelect').empty().trigger("change");
    $('#secondSelect').empty().trigger("change");

    for (var i = 0; i < branches.length; i++) {
        var newOption = new Option(branches[i], branches[i], false, false);

        $('#firstSelect').append(newOption).trigger('change');
        // Somehow we need to create this twice, otherwise it does not work.
        var newOption2 = new Option(branches[i], branches[i], false, false);
        $('#secondSelect').append(newOption2).trigger('change');
    }

    // Select the master branch as first if it exists.
    if (branches.includes("master")) {
        $('#firstSelect').val("master").trigger("change");
    }
}

function load_config() {
    var repos = [];
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
                // for making fielset appear animation
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

        var url = "/compare?unit=weight&ignore_errors=true&threshold=10&method=guess-worst";
        url += "&repo=" + repo;
        url += "&old=" + $('#firstSelect').val();
        url += "&new=" + $('#secondSelect').val();

        if (repo in path_pattern_presets) {
            url += "&path_pattern=" + path_pattern_presets[repo];
        }

        console.log(url);
        window.location.href = url;
    });

    $('#repoSelect').change(function () {
        const repo = $(this).val();

        $('#selectedRepo').val(repo);

        // Load the branches of the repo if not already loaded.
        if (!(repo in branches)) {
            console.log("Loading branches for " + repo);
            $.getJSON("/branches?repo=" + repo, function (d) {
                let data = d['bs'];
                branches[repo] = data;
            }).done(function () {
                populate_branches(branches[repo]);
            });
        } else {
            populate_branches(branches[repo]);
        }
    });
    $('#firstSelect').change(function () {
        $('#selectedFirst').val($(this).val());
    });
    $('#secondSelect').change(function () {
        $('#selectedSecond').val($(this).val());
    });

    $('#fetch').click(function () {
        const repo = $('#repoSelect').val();
        if (repo === null)
            return;
        // Fade the button out within 200 ms
        $('#fetch').fadeOut(200);
        console.log("Fetching repo " + repo);

        $.getJSON("/branches?repo=" + repo + "&fetch=true", function (d) {
            let data = d['bs'];
            branches[repo] = data;
        }).done(function () {
            populate_branches(branches[repo]);
            $('#fetch').fadeIn(200);
        });
    });
});
