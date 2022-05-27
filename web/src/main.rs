use actix_web::{
	get,
	http::header::{CacheControl, CacheDirective},
	middleware,
	middleware::Logger,
	web, App, HttpRequest, HttpResponse, HttpServer,
};
use badge_maker::BadgeBuilder;
use cached::proc_macro::cached;
use clap::Parser;
use dashmap::DashMap;
use lazy_static::lazy_static;
use log::info;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::path::PathBuf;

use swc_core::{
	compare_commits, filter_changes, sort_changes, CompareMethod, CompareParams, FilterParams,
	TotalDiff, Unit, VERSION,
};

mod html;
use html::*;

#[derive(Debug, Parser)]
#[clap(author, version(&VERSION[..]))]
pub(crate) struct MainCmd {
	#[clap(long = "root", short, default_value = "root/")]
	pub root_path: PathBuf,

	#[clap(long, multiple_values = true, default_value = "polkadot")]
	pub repos: Vec<String>,

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

#[derive(Debug, serde::Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct CompareArgs {
	old: String,
	new: String,
	repo: String,
	path_pattern: String,
	ignore_errors: bool,
	threshold: u32,
	unit: Unit,
	method: CompareMethod,
}

#[derive(Debug, serde::Deserialize)]
pub struct VersionArgs {
	is: Option<String>,
}

lazy_static! {
	/// Protects each git repo from concurrent access.
	///
	/// Maps the name of the repo to its path.
	static ref REPOS: DashMap<String, PathBuf> = DashMap::new();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

	let cmd = MainCmd::parse();

	if cmd.repos.is_empty() {
		return Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			"Need at least one value to --repos",
		))
	}
	for repo_name in cmd.repos {
		let path = cmd.root_path.join(&repo_name);
		REPOS.insert(repo_name.clone(), path.clone());
		// Check if the directory exists.
		let _repo = git2::Repository::open(&path).unwrap();
		info!("Exposing repo '{}' at '{}'", &repo_name, path.display());
	}

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
	let args = args.unwrap().into_inner();
	let repos = REPOS.iter().map(|r| r.key().clone()).collect();

	match do_compare_cached(args.clone()) {
		Ok(res) => HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(templates::Compare::render(&res.value, &args, &repos, res.was_cached)),
		Err(e) => http_500(templates::Error::render(&e)),
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

/// Returns a version badge in the style of <https://shields.io>.
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

#[cached(time = 600, result = true, sync_writes = true, with_cached_flag = true)]
fn do_compare_cached(args: CompareArgs) -> Result<cached::Return<TotalDiff>, String> {
	// Call get_mut to acquire an exclusive permit.
	// Assumption tested in `dashmap_exclusive_permit_works`.
	let repo = REPOS
		.get_mut(&args.repo)
		.ok_or(format!("Value '{}' is invalid for argument 'repo'.", &args.repo))?;

	let (new, old) = (args.new.trim(), args.old.trim());
	let (_thresh, unit, method, path_pattern, ignore_errors) =
		(args.threshold, args.unit, args.method, args.path_pattern.trim(), args.ignore_errors);

	let params = CompareParams { method, ignore_errors, unit };
	let mut diff = compare_commits(&repo, old, new, &params, path_pattern, 200)?;
	let filter = FilterParams { threshold: args.threshold as f64, change: None, extrinsic: None };
	diff = filter_changes(diff, &filter);
	sort_changes(&mut diff);

	Ok(cached::Return::new(diff))
}

#[cfg(test)]
mod tests {
	use dashmap::DashMap;

	/// Test my assumption that a shared ref can be used as exclusive permit.
	#[test]
	fn dashmap_exclusive_permit_works() {
		let map = DashMap::new();
		map.insert("foo", "bar");

		// Storing a mutable ref in a shared ref does not decay it.
		{
			let _permit = map.get_mut("foo");
			assert!(map.try_get("foo").is_locked());
		}
		// Meanwhile `get` cannot be used to create an exclusive permit.
		{
			let _permit = map.get("foo");
			assert!(!map.try_get("foo").is_locked());
		}
	}
}
