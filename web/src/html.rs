#![allow(clippy::comparison_chain)] // TODO remove

use actix_web::HttpResponse;

use swc_core::{Percent, RelativeChange, TermChange, Unit};

pub mod templates {
	use super::*;
	use crate::CompareArgs;
	use sailfish::TemplateOnce;
	use swc_core::{CompareMethod, TotalDiff};

	#[derive(TemplateOnce)]
	#[template(path = "root.stpl")]
	pub struct Root {
		repos: Vec<String>,
	}

	#[derive(TemplateOnce)]
	#[template(path = "merge_requests.stpl")]
	pub struct MRs {}

	#[derive(TemplateOnce)]
	#[template(path = "compare.stpl")]
	pub struct Compare<'a> {
		diff: &'a TotalDiff,
		args: &'a CompareArgs,
		repos: &'a Vec<String>,
		was_cached: bool,
	}

	#[derive(TemplateOnce)]
	#[template(path = "error.stpl")]
	pub struct Error<'a> {
		msg: &'a str,
	}

	impl Root {
		pub fn render(repos: Vec<String>) -> String {
			let ctx = Self { repos };
			ctx.render_once().expect("Must render static template; qed")
		}
	}

	impl MRs {
		pub fn render() -> String {
			let ctx = Self {};
			ctx.render_once().expect("Must render static template; qed")
		}
	}

	impl<'a> Compare<'a> {
		pub fn render(
			diff: &'a TotalDiff,
			args: &'a CompareArgs,
			repos: &'a Vec<String>,
			was_cached: bool,
		) -> String {
			let ctx = Self { diff, args, repos, was_cached };
			ctx.render_once().expect("Must render static template; qed")
		}
	}

	impl<'a> Error<'a> {
		pub fn render(msg: &'a str) -> String {
			let ctx = Self { msg };
			ctx.render_once().expect("Must render static template; qed")
		}
	}
}

pub(crate) fn http_500(msg: String) -> HttpResponse {
	HttpResponse::InternalServerError()
		.content_type("text/html; charset=utf-8")
		.body(msg)
}

pub(crate) fn http_200<T>(msg: T) -> HttpResponse
where
	String: std::convert::From<T>,
{
	let msg: String = msg.into();
	HttpResponse::Ok().content_type("text/html; charset=utf-8").body(msg)
}

pub(crate) fn readme_link(name: &str) -> String {
	// Convert the name to a github markdown anchor.
	let anchor = name.to_lowercase().replace(' ', "-");
	format!("{} <a href=\"https://github.com/ggwpez/substrate-weight-compare/#{}\" target=\"_blank\"><sup><small>HELP</small></sup></a>", name, anchor)
}

pub(crate) fn code_link(repo: &str, name: &str, file: &str, rev: &str) -> String {
	format!("<a href=\"https://github.com/paritytech/{}/tree/{}/{}#:~:text=fn {}\" target=\"_blank\"><sup><small>CODE</small></sup></a>", repo, rev, file, name)
}

pub(crate) fn html_color_percent(p: Percent, change: RelativeChange) -> String {
	match change {
		RelativeChange::Changed => {
			if p < 0.0 {
				format!("<p style='color:green'>-{:.2?}%</p>", p.abs())
			} else if p > 0.0 {
				format!("<p style='color:red'>+{:.2?}%</p>", p.abs())
			} else {
				// 0 or NaN
				format!("{:.0?}", p)
			}
		},
		RelativeChange::Unchanged => "<p style='color:gray'>Unchanged</p>".into(),
		RelativeChange::Added => "<p style='color:orange'>Added</p>".into(),
		RelativeChange::Removed => "<p style='color:orange'>Removed</p>".into(),
	}
}

pub(crate) fn html_color_abs(change: &TermChange, unit: Unit) -> String {
	match change.change {
		RelativeChange::Changed => {
			let diff = change.new_v.unwrap() as i128 - change.old_v.unwrap() as i128;
			if diff < 0 {
				format!("<p style='color:green'>-{}</p>", unit.fmt_value(diff.unsigned_abs()))
			} else if diff > 0 {
				format!("<p style='color:red'>+{}</p>", unit.fmt_value(diff.unsigned_abs()))
			} else {
				// 0 or NaN
				format!("{:.0?}", diff)
			}
		},
		RelativeChange::Unchanged => "<p style='color:gray'>Unchanged</p>".into(),
		RelativeChange::Added => "<p style='color:orange'>Added</p>".into(),
		RelativeChange::Removed => "<p style='color:orange'>Removed</p>".into(),
	}
}

/// Converts a relative change to an absolute value to make it sortable in html.
///
/// Note: Undefined for values > i128::MAX or < i128::MIN.
fn order_percent(change: &TermChange) -> i128 {
	match change.change {
		// This only considers the first three digits of the percent since the UI only shows these.
		RelativeChange::Changed => (change.percent * 1000.0) as i128,
		RelativeChange::Unchanged => 0,
		RelativeChange::Added => i128::MAX,
		RelativeChange::Removed => i128::MIN,
	}
}

fn order_abs(change: &TermChange) -> i128 {
	match change.change {
		RelativeChange::Changed => change.new_v.unwrap() as i128 - change.old_v.unwrap() as i128,
		RelativeChange::Unchanged => 0,
		RelativeChange::Added => i128::MAX,
		RelativeChange::Removed => i128::MIN,
	}
}
