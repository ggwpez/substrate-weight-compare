#![allow(dead_code)] // The caching crate produces "unused" functions…

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
use std::{
	path::{Path, PathBuf},
	process::Command,
	sync::{Arc, Mutex},
};

use subweight_core::{
	compare_commits, filter_changes, sort_changes, CompareMethod, CompareParams, Dimension,
	FilterParams, TotalDiff, VERSION,
};

mod git;
mod html;
use html::*;

#[derive(Debug, Parser, Clone)]
#[clap(author, version(&VERSION[..]))]
pub(crate) struct MainCmd {
	#[clap(long = "root", short, default_value = "root/")]
	pub root_path: PathBuf,

	#[clap(long = "static", short, default_value = "web/static")]
	pub static_path: PathBuf,

	#[clap(long, num_args = 0.., default_value = "polkadot")]
	pub repos: Vec<String>,

	#[clap(long, short, default_value = "127.0.0.1")]
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
	unit: Dimension,
	git_pull: Option<bool>,
	method: CompareMethod,
}

#[derive(Debug, serde::Deserialize)]
pub struct VersionArgs {
	is: Option<String>,
}

#[derive(Clone)]
pub struct Repo {
	name: String,
	path: PathBuf,
	organization: String,
	operation: Arc<Mutex<()>>,
}

lazy_static! {
	/// Repository metadata; never hold a map guard during Git or comparison work.
	///
	/// Maps the name of the repo to its origin-name and path.
	static ref REPOS: DashMap<String, Repo> = DashMap::new();
	static ref HEAVY_JOBS: Arc<tokio::sync::Semaphore> = Arc::new(tokio::sync::Semaphore::new(4));
	static ref CONFIG: MainCmd = MainCmd::parse();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	if std::env::var("RUST_BACKTRACE").is_err() {
		std::env::set_var("RUST_BACKTRACE", "1");
	}
	env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
	let cmd = CONFIG.clone();
	let static_path = cmd.static_path.into_os_string();

	if cmd.repos.is_empty() {
		return Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			"Need at least one value to --repos",
		));
	}

	if !cmd.root_path.exists() {
		return Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			format!("Root path '{}' does not exist", cmd.root_path.display()),
		));
	}

	for repo_name in cmd.repos {
		// TODO check that the folder exists
		let path = cmd.root_path.join(&repo_name);

		if !path.exists() {
			return Err(std::io::Error::new(
				std::io::ErrorKind::Other,
				format!("Repo directory '{}' does not exist", path.display()),
			));
		}

		let organization = git::get_origin_org(&path).map_err(|e| {
			std::io::Error::new(
				std::io::ErrorKind::Other,
				format!("Failed to get origin of {}: {}", repo_name, e),
			)
		})?;
		REPOS.insert(
			repo_name.clone(),
			Repo {
				name: repo_name.clone(),
				path: path.clone(),
				organization: organization.clone(),
				operation: Arc::new(Mutex::new(())),
			},
		);
		// Check if the repo directory exists.
		if !path.exists() {
			return Err(std::io::Error::new(
				std::io::ErrorKind::Other,
				format!("Repo directory '{}' does not exist", path.display()),
			));
		}
		info!("Exposing repo '{}/{}' at '{}'", &organization, &repo_name, path.display());
	}
	// check that static_path is a dir
	if !Path::new(&static_path).is_dir() {
		return Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			format!("Web root path '{:?}' is not a directory", static_path),
		));
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

	let permit = HEAVY_JOBS.clone().try_acquire_owned().map_err(|_| {
		actix_web::error::ErrorServiceUnavailable("Git/comparison capacity is busy; retry later")
	})?;
	let args = args.into_inner();
	let branch = web::block(move || {
		let _permit = permit;
		list_branches(args)
	})
	.await
	.map_err(actix_web::error::ErrorInternalServerError)?
	.map_err(actix_web::error::ErrorInternalServerError)?;

	#[derive(Serialize)]
	struct Branches {
		branch: Vec<(String, String)>,
	}

	let obj = Branches { branch };
	Ok(web::Json(obj))
}

fn list_branches(args: BranchArgs) -> std::io::Result<Vec<(String, String)>> {
	let repo = REPOS.get(&args.repo).map(|r| r.value().clone()).ok_or_else(|| {
		std::io::Error::new(std::io::ErrorKind::Other, format!("Unknown repo '{}'", args.repo))
	})?;
	let _operation = repo
		.operation
		.lock()
		.map_err(|_| std::io::Error::other("Repository lock poisoned"))?;
	if args.fetch.unwrap_or_default() {
		info!("Fetching branches for '{}'", &args.repo);
		// Fetch all tags and branches from the repo by spawning a git command
		// and parsing the output.
		let output = subweight_core::command::git_output(
			Command::new("git")
				.arg("fetch")
				.arg("--all")
				.arg("--prune")
				.arg("--tags")
				.current_dir(repo.path.deref()),
		)
		.map_err(|e| {
			std::io::Error::new(
				std::io::ErrorKind::Other,
				format!("Failed to fetch branches: '{}'", e),
			)
		})?;
		if !output.status.success() {
			let err = String::from_utf8(output.stderr).unwrap();
			log::error!("Failed to fetch branches: '{}'", &err);

			return Err(std::io::Error::new(
				std::io::ErrorKind::Other,
				format!("Failed to fetch branches: '{}'", &err),
			)
			.into());
		}
	}

	// Spawn a git command and return all branches
	let output = subweight_core::command::git_output(
		Command::new("git")
			.args(["ls-remote", "--tags", "--heads"])
			.current_dir(repo.path.deref()),
	)?;
	if !output.status.success() {
		let err = String::from_utf8(output.stderr).unwrap();
		log::error!("Failed to list branches: {}", &err);
		return Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			format!("Failed to list branches: {}", &err),
		)
		.into());
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

	Ok(branch)
}

#[get("/compare")]
async fn compare(req: HttpRequest) -> HttpResponse {
	let args = web::Query::<CompareArgs>::from_query(req.query_string());
	if let Err(err) = args {
		return http_500(templates::Error::render(&err.to_string()));
	}
	let mut args = args.unwrap().into_inner();
	// HTML decode the new and old branch names. TODO clean this up
	args.new = html_escape::decode_html_entities(&args.new).to_string();
	args.old = html_escape::decode_html_entities(&args.old).to_string();
	args.path_pattern = html_escape::decode_html_entities(&args.path_pattern).to_string();

	let repos = REPOS.iter().map(|r| r.key().clone()).collect();
	// TODO dont do two lookups here…
	let organization = REPOS.get(&args.repo).map(|r| r.organization.clone());

	if organization.is_none() {
		return http_500(templates::Error::render(&format!(
			"Unknown repo organization '{}'",
			&args.repo
		)));
	}

	let permit = match HEAVY_JOBS.clone().try_acquire_owned() {
		Ok(permit) => permit,
		Err(_) =>
			return HttpResponse::ServiceUnavailable()
				.body("Git/comparison capacity is busy; retry later"),
	};
	let result = web::block(move || {
		let _permit = permit;
		do_compare_cached(args.clone())
			.map(|res| {
				templates::Compare::render(
					&res.value,
					&args,
					organization.unwrap(),
					&repos,
					res.was_cached,
				)
			})
			.map_err(|e| e.to_string())
	})
	.await;
	match result {
		Ok(Ok(html)) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
		Ok(Err(e)) => http_500(templates::Error::render(&e)),
		Err(e) => http_500(templates::Error::render(&e.to_string())),
	}
}

#[derive(Deserialize)]
struct MrArgs {}

/// This endpoint is a two-in one. If no repo is passed,
#[get("/compare-mr")]
async fn compare_mrs(_req: HttpRequest) -> HttpResponse {
	let repos = REPOS.iter().map(|r| r.value().clone()).collect();
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
	let current = subweight_core::VERSION.clone();

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
		.message(&subweight_core::VERSION)
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
	// Serialize repository mutation without locking metadata or HTTP workers.
	let repo = REPOS
		.get(&args.repo)
		.map(|r| r.value().clone())
		.ok_or(format!("Value '{}' is invalid for argument 'repo'.", &args.repo))?;

	let _operation = repo.operation.lock().map_err(|_| "Repository lock poisoned")?;

	let (new, old) = (args.new.trim(), args.old.trim());
	let (_thresh, unit, method, path_pattern, ignore_errors, git_pull) = (
		args.threshold,
		args.unit,
		args.method,
		args.path_pattern.trim(),
		args.ignore_errors,
		args.git_pull.unwrap_or(true),
	);

	let params =
		CompareParams { method, ignore_errors, unit, git_pull, offline: false, git_force: true };
	let filter = FilterParams {
		threshold: args.threshold as f64,
		change: None,
		pallet: args.pallet,
		extrinsic: args.extrinsic,
	};

	let mut diff = compare_commits(&repo.path, old, new, &params, &filter, path_pattern, 6000)?;
	diff = filter_changes(diff, &filter);
	sort_changes(&mut diff);

	Ok(cached::Return::new(diff))
}

#[cfg(test)]
mod tests {
	use super::*;
	use actix_web::{http::StatusCode, test};
	use std::time::Duration;

	#[actix_web::test]
	async fn stalled_repository_work_does_not_block_pages_and_excess_work_is_rejected() {
		let repo = Repo {
			name: "stalled-test".into(),
			path: PathBuf::new(),
			organization: "test".into(),
			operation: Arc::new(Mutex::new(())),
		};
		REPOS.insert(repo.name.clone(), repo.clone());
		let permits = HEAVY_JOBS.clone().acquire_many_owned(4).await.unwrap();
		let (started_tx, started_rx) = tokio::sync::oneshot::channel();
		let (release_tx, release_rx) = std::sync::mpsc::channel();
		let job = actix_web::rt::spawn(async move {
			web::block(move || {
				let _permit = permits;
				let _lock = repo.operation.lock().unwrap();
				started_tx.send(()).unwrap();
				// Bound the fixture even if the test fails.
				let _ = release_rx.recv_timeout(Duration::from_secs(3));
			})
			.await
			.unwrap();
		});
		started_rx.await.unwrap();
		let app =
			test::init_service(App::new().service(root).service(version).service(branches)).await;
		for (uri, expected) in [
			("/", StatusCode::OK),
			("/version", StatusCode::OK),
			("/branches?repo=stalled-test", StatusCode::SERVICE_UNAVAILABLE),
		] {
			let response = actix_web::rt::time::timeout(
				Duration::from_millis(500),
				test::call_service(&app, test::TestRequest::get().uri(uri).to_request()),
			)
			.await
			.expect("HTTP worker must remain responsive");
			assert_eq!(response.status(), expected);
		}
		release_tx.send(()).unwrap();
		job.await.unwrap();
		assert_eq!(HEAVY_JOBS.available_permits(), 4);
		REPOS.remove("stalled-test");
	}
}
