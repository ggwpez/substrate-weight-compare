use actix_files as fs;
use actix_web::{
	get,
	http::header::{CacheControl, CacheDirective},
	middleware,
	middleware::Logger,
	web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use badge_maker::BadgeBuilder;
use cached::proc_macro::cached;
use clap::Parser;
use dashmap::DashMap;
use lazy_static::{__Deref, lazy_static};
use log::info;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, process::Command};

use swc_core::{
	compare_commits, filter_changes, sort_changes, CompareMethod, CompareParams, FilterParams,
	TotalDiff, Unit, VERSION,
};

mod html;
use html::*;

#[derive(Debug, Parser, Clone)]
#[clap(author, version(&VERSION[..]))]
pub(crate) struct MainCmd {
	#[clap(long = "root", short, default_value = "root/")]
	pub root_path: PathBuf,

	#[clap(long = "static", short, default_value = "web/static")]
	pub static_path: PathBuf,

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
	extrinsic: Option<String>,
	pallet: Option<String>,
	ignore_errors: bool,
	threshold: u32,
	unit: Unit,
	git_pull: Option<bool>,
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
	static ref CONFIG: MainCmd = MainCmd::parse();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
	let cmd = CONFIG.clone();
	let static_path = cmd.static_path.into_os_string();

	if cmd.repos.is_empty() {
		return Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			"Need at least one value to --repos",
		))
	}
	for repo_name in cmd.repos {
		let path = cmd.root_path.join(&repo_name);
		REPOS.insert(repo_name.clone(), path.clone());
		// Check if the repo directory exists.
		if !path.exists() {
			return Err(std::io::Error::new(
				std::io::ErrorKind::Other,
				format!("Repo directory '{}' does not exist", path.display()),
			))
		}
		info!("Exposing repo '{}' at '{}'", &repo_name, path.display());
	}

	let endpoint = format!("{}:{}", cmd.endpoint, cmd.port);
	info!("Listening to http://{}", endpoint);

	let server = HttpServer::new(move || {
		App::new()
			.wrap(middleware::Compress::default())
			.wrap(Logger::new("%a %r %s %b %{Referer}i %Ts"))
			.service(fs::Files::new("/static", &static_path).show_files_listing())
			.service(compare)
			.service(version_badge)
			.service(version)
			.service(root)
			.service(branches)
			.service(compare_mrs)
			.service(compare_commit)
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
	let repos = REPOS.iter().map(|r| r.key().clone()).collect();

	http_200(templates::Root::render(repos))
}

// TODO
#[get("/compare-commit")]
async fn compare_commit() -> HttpResponse {
	let repos = REPOS.iter().map(|r| r.key().clone()).collect();

	http_200(templates::Root::render(repos))
}

/// Returns supported repositories.
#[get("/repos")]
async fn repositories() -> Result<impl Responder> {
	let repos = REPOS.iter().map(|r| r.key().clone()).collect();

	#[derive(Serialize)]
	struct Info {
		repos: Vec<String>,
	}

	let obj = Info { repos };
	Ok(web::Json(obj))
}

#[derive(Deserialize)]
struct BranchArgs {
	repo: String,
	fetch: Option<bool>,
}

/// Returns the available branches for the repositories.
#[get("/branches")]
async fn branches(req: HttpRequest) -> Result<impl Responder> {
	let args = web::Query::<BranchArgs>::from_query(req.query_string()).map_err(|e| {
		std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to parse query: {}", e))
	})?;

	let path = REPOS.get(&args.repo).ok_or_else(|| {
		std::io::Error::new(std::io::ErrorKind::Other, format!("Unknown repo '{}'", args.repo))
	})?;
	if args.fetch.unwrap_or_default() {
		info!("Fetching branches for '{}'", &args.repo);
		// Fetch all tags and branches from the repo by spawning a git command
		// and parsing the output.
		let output = Command::new("git")
			.arg("fetch")
			.arg("--all")
			.arg("--prune")
			.arg("--tags")
			.current_dir(path.deref())
			.output()
			.map_err(|e| {
				std::io::Error::new(
					std::io::ErrorKind::Other,
					format!("Failed to fetch branches: {}", e),
				)
			})?;
		if !output.status.success() {
			let err = String::from_utf8(output.stderr).unwrap();
			log::error!("Failed to fetch branches: {}", &err);

			return Err(std::io::Error::new(
				std::io::ErrorKind::Other,
				format!("Failed to fetch branches: {}", &err),
			)
			.into())
		}
	}

	// Spawn a git command and return all branches
	let output = Command::new("git")
		.args(&["ls-remote", "--tags", "--heads"])
		.current_dir(path.deref())
		.output()?;
	if !output.status.success() {
		let err = String::from_utf8(output.stderr).unwrap();
		log::error!("Failed to list branches: {}", &err);
		return Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			format!("Failed to list branches: {}", &err),
		)
		.into())
	}
	let stdout = String::from_utf8_lossy(&output.stdout);
	// Collect all branches and remove the leading refs/heads/
	let branch = stdout
		.lines()
		// Some tags contain weird stuff like {} or ^, let's filter those out.
		.filter(|l| !l.contains('{') && !l.contains('^'))
		.map(|l| l.replace("refs/heads/", ""))
		.map(|l| l.replace("refs/tags/", ""))
		// Split at whitespace and use the first part as the branch name
		// and the second as commit hash.
		.map(|l| {
			let splits = l.split_whitespace().collect::<Vec<&str>>();
			(splits[1].to_string(), splits[0][..12].to_string())
		})
		.collect::<Vec<(String, String)>>();

	#[derive(Serialize)]
	struct Branches {
		branch: Vec<(String, String)>,
	}

	let obj = Branches { branch };
	Ok(web::Json(obj))
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
		Err(e) => http_500(templates::Error::render(&e.to_string())),
	}
}

#[derive(Deserialize)]
struct MrArgs {}

/// This endpoint is a two-in one. If no repo is passed,
#[get("/compare-mr")]
async fn compare_mrs(_req: HttpRequest) -> HttpResponse {
	let repos = REPOS.iter().map(|r| r.key().clone()).collect();
	http_200(templates::MRs::render(repos))
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
fn do_compare_cached(
	args: CompareArgs,
) -> Result<cached::Return<TotalDiff>, Box<dyn std::error::Error>> {
	// Call get_mut to acquire an exclusive permit.
	// Assumption tested in `dashmap_exclusive_permit_works`.
	let repo = REPOS
		.get_mut(&args.repo)
		.ok_or(format!("Value '{}' is invalid for argument 'repo'.", &args.repo))?;

	let (new, old) = (args.new.trim(), args.old.trim());
	let (_thresh, unit, method, path_pattern, ignore_errors, git_pull) = (
		args.threshold,
		args.unit,
		args.method,
		args.path_pattern.trim(),
		args.ignore_errors,
		args.git_pull.unwrap_or(true),
	);

	let params = CompareParams { method, ignore_errors, unit, git_pull };
	let filter = FilterParams {
		threshold: args.threshold as f64,
		change: None,
		pallet: args.pallet,
		extrinsic: args.extrinsic,
	};

	let mut diff = compare_commits(&repo, old, new, &params, &filter, path_pattern, 200)?;
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
