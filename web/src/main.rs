use actix_web::{
	get,
	http::header::{CacheControl, CacheDirective},
	middleware,
	middleware::Logger,
	web, App, HttpRequest, HttpResponse, HttpServer,
};
use badge_maker::BadgeBuilder;
use clap::Parser;
use lazy_static::lazy_static;
use log::info;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::{path::PathBuf, sync::Mutex};

use swc_core::{
	compare_commits, filter_changes, sort_changes, CompareMethod, CompareParams, FilterParams, VERSION,
};

mod html;
use html::*;

#[derive(Debug, Parser)]
#[clap(author, version(&VERSION[..]))]
pub(crate) struct MainCmd {
	#[clap(long, short, default_value = "repos/substrate")]
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

#[derive(Debug, serde::Deserialize)]
pub struct CompareArgs {
	old: String,
	new: String,
	path_pattern: String,
	ignore_errors: bool,
	threshold: f64,
	method: CompareMethod,
}

#[derive(Debug, serde::Deserialize)]
pub struct VersionArgs {
	is: Option<String>,
}

lazy_static! {
	/// Singleton mutex to protect the git repo from concurrent access.
	///
	/// Contains the path to the repo.
	static ref REPO: Mutex<Option<PathBuf>> = Mutex::new(None);
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
			.service(version_badge)
			.service(version)
			.service(root)
	})
	.workers(4);

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

#[get("/")]
async fn root() -> HttpResponse {
	http_200(templates::Root::render())
}

#[get("/compare")]
async fn compare(req: HttpRequest) -> HttpResponse {
	let args = web::Query::<CompareArgs>::from_query(req.query_string());
	if let Err(err) = args {
		return http_500(templates::Error::render(&err.to_string()))
	}

	match do_compare(args.unwrap().into_inner()) {
		Ok(res) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(res),
		Err(e) => http_500(e),
	}
}

/// Exposes version information for automatic deployments.
///
/// Has two modi operandi:
/// - `/version` returns the current version.
/// - `/version?is=1.2` can be used to check if the server runs a specific version.
/// Returns codes 200 or 500.
#[get("/version")]
async fn version(web::Query(args): web::Query<VersionArgs>) -> HttpResponse {
	let current = swc_core::VERSION.clone();

	if let Some(version) = args.is {
		// Hack: + becomes a space in query params, so just replace it…
		if current == version || current.replace('+', " ") == version {
			http_200("Version check passed")
		} else {
			http_500(format!("Version check failed: '{}' vs '{}'", current, version))
		}
	} else {
		HttpResponse::Ok()
			.insert_header(CacheControl(vec![
				CacheDirective::NoCache,
				CacheDirective::Public,
				CacheDirective::MaxAge(600u32),
			]))
			.content_type("text/html; charset=utf-8")
			.body(current)
	}
}

#[get("/version/badge")]
async fn version_badge() -> HttpResponse {
	let svg = BadgeBuilder::new()
		.label("Deployed")
		.message(&*swc_core::VERSION)
		.color_parse("#33B5E5")
		.build()
		.expect("Must build svg")
		.svg();

	HttpResponse::Ok()
		.insert_header(CacheControl(vec![
			CacheDirective::NoCache,
			CacheDirective::Public,
			CacheDirective::MaxAge(600u32),
		]))
		.content_type("image/svg+xml")
		.body(svg)
}

fn do_compare(args: CompareArgs) -> Result<String, String> {
	let repo_guard = match REPO.lock() {
		Ok(guard) => guard,
		Err(poisoned) => poisoned.into_inner(),
	};
	let repo_path: PathBuf =
		repo_guard.as_ref().ok_or_else(|| "Could not lock mutex".to_string())?.clone();

	let (new, old) = (args.new.trim(), args.old.trim());
	let (_thresh, method, path_pattern, ignore_errors) =
		(args.threshold, args.method, args.path_pattern.trim(), args.ignore_errors);

	let params = CompareParams { method, ignore_errors };
	let mut diff = compare_commits(&repo_path, old, new, &params, path_pattern, 200)?;
	let filter = FilterParams {
		threshold: args.threshold,
		change: None,
	};
	diff = filter_changes(diff, &filter);
	sort_changes(&mut diff);

	Ok(templates::Compare::render(&diff, &args))
}
