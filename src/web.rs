use glob::glob;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::{path::PathBuf, str::FromStr};
use substrate_weight_compare::*;

use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use git2::*;

lazy_static! {
    static ref MUTEX: Mutex<()> = Mutex::new(());
}

#[get("/compare/{old}/{new}/{thresh}")]
async fn compare(commits: web::Path<(String, String, Option<String>)>) -> impl Responder {
    let _guard = MUTEX.lock();
    let (old, new, thresh) = (
        commits.0.trim(),
        commits.1.trim(),
        commits.2.clone().unwrap_or_else(|| "30".into()),
    );

    if let Err(err) = checkout("test_data/polkadot_old".into(), old) {
        return HttpResponse::InternalServerError().body(format!("{:?}", err));
    }
    if let Err(err) = checkout("test_data/polkadot_new".into(), new) {
        return HttpResponse::InternalServerError().body(format!("{:?}", err));
    }
    let old_paths = list_files("test_data/polkadot_old/runtime/polkadot/src/weights/*.rs");
    let new_paths = list_files("test_data/polkadot_new/runtime/polkadot/src/weights/*.rs");

    let params = CompareParams {
        old: old_paths,
        new: new_paths,
        blacklist_file: vec!["mod.rs".into()],
        threshold: thresh.parse().unwrap(),
    };
    let diff = compare_files(&params);
    let per_extrinsic = extract_changes(&params, diff);

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

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
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

fn list_files(regex: &str) -> Vec<PathBuf> {
    let files = glob(regex).unwrap();
    files.map(|f| f.unwrap()).collect()
}

fn checkout(path: PathBuf, commit_hash: &str) -> Result<(), git2::Error> {
    let repo = Repository::open(path)?;

    let refname = commit_hash; // or a tag (v0.1.1) or a commit (8e8128)
    let (object, reference) = repo.revparse_ext(refname)?;

    repo.checkout_tree(&object, None)?;

    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repo.set_head_detached(object.id()),
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

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(index)
}

#[get("/")]
async fn root() -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", "/compare"))
        .finish()
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let endpoint = "localhost:8080";
    println!("Listening to http://{}", endpoint);

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
