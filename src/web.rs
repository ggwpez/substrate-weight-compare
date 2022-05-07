use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use lazy_static::lazy_static;
use log::info;
use std::{path::PathBuf, str::FromStr, sync::Mutex};

use swc::{compare_commits, VERSION};

#[derive(Debug, Parser)]
#[clap(author, version(&VERSION[..]))]
pub(crate) struct MainCmd {
	#[clap(long, short, default_value = "repos/polkadot")]
	pub repo: PathBuf,

	#[clap(long, short, default_value = "localhost")]
	pub endpoint: String,

	#[clap(long, short, default_value = "8080")]
	pub port: u16,
}

lazy_static! {
	static ref REPO: Mutex<Option<PathBuf>> = Mutex::new(None);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	env_logger::init_from_env(
		env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
	);

	let cmd = MainCmd::parse();
	*REPO.lock().unwrap() = Some(cmd.repo);
	let endpoint = format!("{}:{}", cmd.endpoint, cmd.port);
	info!("Listening to http://{}", endpoint);

	HttpServer::new(|| {
		App::new()
			.wrap(middleware::Compress::default())
			.service(compare)
			.service(compare_index)
			.service(root)
	})
	.bind(endpoint)?
	.run()
	.await
}

#[get("/compare/{old}/{new}/{thresh}")]
async fn compare(commits: web::Path<(String, String, Option<String>)>) -> impl Responder {
	let repo_guard = REPO.lock().unwrap();
	let repo_path: PathBuf = repo_guard.as_ref().unwrap().clone();

	let (old, new, thresh) =
		(commits.0.trim(), commits.1.trim(), commits.2.clone().unwrap_or_else(|| "30".into()));

	let per_extrinsic = compare_commits(&repo_path, old, new, thresh.parse().unwrap()).unwrap();

	let mut output = String::from_str(
		"<table><tr><th>Extrinsic</th><th>Old [ns]</th><th>New [ns]</th><th>Diff [%]</th></tr>",
	)
	.unwrap();
	for change in per_extrinsic {
		output.push_str(&format!(
			"<tr><td>{}::{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
			change.file,
			change.name,
			change.old,
			change.new,
			html_color_percent(change.change),
		));
	}
	output.push_str("</table>");

	HttpResponse::Ok().content_type("text/html; charset=utf-8").body(format!(
		"Compared Polkadot {old} (old) to {new} (new) with {thresh}% threshold:</br>{output}"
	))
}

// Use html colors
fn html_color_percent(p: f64) -> String {
	if p < 0.0 {
		format!("<p style='color:green'>-{:.2?}</p>", p.abs())
	} else if p > 0.0 {
		format!("<p style='color:red'>+{:.2?}</p>", p.abs())
	} else {
		// 0 or NaN
		format!("{:.0?}", p)
	}
}

#[get("/compare")]
async fn compare_index() -> HttpResponse {
	let index = r#"
    <h1>Examples:</h1>
    <ul>
        <li>
            <a href='/compare/20467ccea1ae3bc89362d3980fde9383ce334789/master/30'>Example #1</a>
        </li>
        <li>
            <a href='/compare/v0.9.18/v0.9.19/10'>Example #2</a>
        </li>
    </ul>
    "#;

	HttpResponse::Ok().content_type("text/html; charset=utf-8").body(index)
}

#[get("/")]
async fn root() -> HttpResponse {
	HttpResponse::Found().append_header(("Location", "/compare")).finish()
}
