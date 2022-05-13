use actix_web::{get, middleware, middleware::Logger, web, App, HttpResponse, HttpServer};
use clap::Parser;
use lazy_static::lazy_static;
use log::info;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::{cmp::Ordering, path::PathBuf, sync::Mutex};

use swc_core::{
	compare_commits, fmt_weight, CompareMethod, Percent, RelativeChange, TotalDiff, VERSION,
};

#[derive(Debug, Parser)]
#[clap(author, version(&VERSION[..]))]
pub(crate) struct MainCmd {
	#[clap(long, short, default_value = "repos/polkadot")]
	pub repo: PathBuf,

	#[clap(long, short, default_value = "localhost")]
	pub endpoint: String,

	#[clap(long, short, default_value = "8080")]
	pub port: u16,

	/// PEM format cert.
	#[clap(long, requires("key"))]
	pub cert: Option<String>,

	/// PEM format key.
	#[clap(long, requires("cert"))]
	pub key: Option<String>,
}

lazy_static! {
	static ref REPO: Mutex<Option<PathBuf>> = Mutex::new(None);
}

use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "compare.stpl")]
struct CompareTemplate {
	diff: TotalDiff,
	args: CompareArgs,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

	let cmd = MainCmd::parse();
	*REPO.lock().unwrap() = Some(cmd.repo);
	let endpoint = format!("{}:{}", cmd.endpoint, cmd.port);
	info!("Listening to http://{}", endpoint);

	let server = HttpServer::new(|| {
		App::new()
			.wrap(middleware::Compress::default())
			.wrap(Logger::new("%a %r %s %b %{Referer}i %Ts"))
			.service(compare)
			.service(compare_index)
			.service(root)
	})
	.workers(1); // Comparing commits cannot be parallelized.

	let bound_server = if let Some(cert) = cmd.cert {
		let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
		builder
			.set_private_key_file(cmd.key.expect("Checked by clap"), SslFiletype::PEM)
			.unwrap();
		builder.set_certificate_chain_file(cert).unwrap();
		server.bind_openssl(endpoint, builder)
	} else {
		server.bind(endpoint)
	};

	bound_server?.run().await
}

#[derive(Debug, serde::Deserialize)]
pub struct CompareArgs {
	old: String,
	new: String,
	path_pattern: String,
	ignore_errors: bool,
	threshold: String,
	method: CompareMethod,
}

fn readme_link(name: &str) -> String {
	// Convert the name to a github markdown anchor.
	let anchor = name.to_lowercase().replace(" ", "-");
	format!("{} <a href=\"https://github.com/ggwpez/substrate-weight-compare/README.md#{}\" target=\"_blank\"><sup><small>HELP</small></sup></a>", name, anchor)
}

fn code_link(name: &str, file: &str, rev: &str) -> String {
	format!("<a href=\"https://github.com/paritytech/polkadot/tree/{}/{}#:~:text=fn {}\" target=\"_blank\"><sup><small>CODE</small></sup></a>", rev, file, name)
}

fn do_compare(args: CompareArgs) -> Result<String, String> {
	let repo_guard = match REPO.lock() {
		Ok(guard) => guard,
		Err(poisoned) => poisoned.into_inner(),
	};
	let repo_path: PathBuf = repo_guard.as_ref().ok_or(format!("Could not lock mutex"))?.clone();

	let (new, old) = (args.new.trim(), args.old.trim());
	let (thresh, method, path_pattern, ignore_errors) =
		(args.threshold.clone(), args.method, args.path_pattern.trim(), args.ignore_errors);

	let mut diff = compare_commits(
		&repo_path,
		old,
		new,
		thresh.parse().map_err(|e| format!("could not parse threshold: {:?}", e))?,
		method,
		&path_pattern,
		ignore_errors,
		200,
	)?;
	diff.sort_by(|a, b| {
		let ord = a.change.change.cmp(&b.change.change).reverse();
		if ord == Ordering::Equal {
			if a.change.percent > b.change.percent {
				Ordering::Greater
			} else if a.change.percent == b.change.percent {
				Ordering::Equal
			} else {
				Ordering::Less
			}
		} else {
			ord
		}
	});

	let ctx = CompareTemplate { diff, args };
	ctx.render_once().map_err(|e| format!("Could not render template: {}", e))
}

#[get("/compare")]
async fn compare(args: web::Query<CompareArgs>) -> HttpResponse {
	match do_compare(args.into_inner()) {
		Ok(res) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(res),
		Err(e) => HttpResponse::InternalServerError()
			.content_type("text/html; charset=utf-8")
			.body(format!("<pre>Error: {:?}</pre>", e)),
	}
}

// Use html colors
fn html_color_percent(p: Percent, change: RelativeChange) -> String {
	match change {
		RelativeChange::Change =>
			if p < 0.0 {
				format!("<p style='color:green'>-{:.2?}</p>", p.abs())
			} else if p > 0.0 {
				format!("<p style='color:red'>+{:.2?}</p>", p.abs())
			} else {
				// 0 or NaN
				format!("{:.0?}", p)
			},
		RelativeChange::Unchanged => "<p style='color:gray'>Unchanged</p>".into(),
		RelativeChange::Added => "<p style='color:orange'>Added</p>".into(),
		RelativeChange::Removed => "<p style='color:orange'>Removed</p>".into(),
	}
}

#[get("/")]
async fn compare_index() -> HttpResponse {
	let index = r#"
    <h1>Examples:</h1>
    <ul>
        <li>:?
            <a href='/compare/20467ccea1ae3bc89362d3980fde9383ce334789/master/30/polkadot'>Example #1</a>
        </li>
        <li>
            <a href='/compare/v0.9.18/v0.9.19/10/kusama'>Example #2</a>
        </li>
    </ul>
    "#;

	HttpResponse::Ok().content_type("text/html; charset=utf-8").body(index)
}

#[get("/")]
async fn root() -> HttpResponse {
	HttpResponse::Found().append_header(("Location", "/compare")).finish()
}
