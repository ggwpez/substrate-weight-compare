use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use lazy_static::lazy_static;
use std::str::FromStr;
use std::sync::Mutex;
use swc::{compare_commits, VERSION};

#[derive(Debug, Parser)]
#[clap(author, version(&VERSION[..]))]
pub(crate) struct WebCmd {}

lazy_static! {
    static ref MUTEX: Mutex<()> = Mutex::new(());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _cmd = WebCmd::parse();
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

#[get("/compare/{old}/{new}/{thresh}")]
async fn compare(commits: web::Path<(String, String, Option<String>)>) -> impl Responder {
    let _guard = MUTEX.lock();
    let (old, new, thresh) = (
        commits.0.trim(),
        commits.1.trim(),
        commits.2.clone().unwrap_or_else(|| "30".into()),
    );
    let blacklist_file = vec!["mod.rs".into()];

    let per_extrinsic = compare_commits(old, new, thresh.parse().unwrap(), blacklist_file).unwrap();

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
