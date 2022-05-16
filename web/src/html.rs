use actix_web::HttpResponse;

use swc_core::{Percent, RelativeChange};

pub mod templates {
	use super::*;
	use crate::CompareArgs;
	use sailfish::TemplateOnce;
	use swc_core::{fmt_weight, CompareMethod, TotalDiff};

	#[derive(TemplateOnce)]
	#[template(path = "root.stpl")]
	pub struct Root {}

	#[derive(TemplateOnce)]
	#[template(path = "compare.stpl")]
	pub struct Compare<'a> {
		diff: &'a TotalDiff,
		args: &'a CompareArgs,
	}

	#[derive(TemplateOnce)]
	#[template(path = "error.stpl")]
	pub struct Error<'a> {
		msg: &'a str,
	}

	impl Root {
		pub fn render() -> String {
			let ctx = Self {};
			ctx.render_once().expect("Must render static template; qed")
		}
	}

	impl<'a> Compare<'a> {
		pub fn render(diff: &'a TotalDiff, args: &'a CompareArgs) -> String {
			let ctx = Self { diff, args };
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

pub(crate) fn code_link(name: &str, file: &str, rev: &str) -> String {
	format!("<a href=\"https://github.com/paritytech/polkadot/tree/{}/{}#:~:text=fn {}\" target=\"_blank\"><sup><small>CODE</small></sup></a>", rev, file, name)
}

pub(crate) fn html_color_percent(p: Percent, change: RelativeChange) -> String {
	match change {
		RelativeChange::Change => {
			if p < 0.0 {
				format!("<p style='color:green'>-{:.2?}</p>", p.abs())
			} else if p > 0.0 {
				format!("<p style='color:red'>+{:.2?}</p>", p.abs())
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
