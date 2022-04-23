use clap::Parser;
use log::*;
use std::collections::BTreeMap as Map;
use std::collections::BTreeSet as Set;
use std::io::Read;
use std::path::PathBuf;
use syn::{Expr, ImplItem, ImplItemMethod, Item, Lit, Stmt, Type};

// 1000 weight
type WeightNs = u64;

#[derive(Debug, clap::Parser)]
struct MainCmd {
    /// The old weight files.
    #[clap(long, required(true), multiple_values(true))]
    pub old: Vec<PathBuf>,

    /// The new weight files.
    #[clap(long, required(true), multiple_values(true))]
    pub new: Vec<PathBuf>,

    /// Skips files that end with any of these strings.
    #[clap(long, multiple_values(true), default_values = &["mod.rs"])]
    pub blacklist_file: Vec<String>,

    #[clap(long, value_name = "PERCENT", default_value = "5")]
    pub threshold: Percent,
}

type Percent = f64;

enum ExtrinsicDiff {
    Same,
    Added,
    Removed,
    Change(WeightNs, WeightNs, Percent),
}

// File -> Extrinsic -> Diff
type TotalDiff = Map<String, Map<String, ExtrinsicDiff>>;

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let args = MainCmd::parse();

    let olds = parse_files(&args.old, args.blacklist_file.clone()).unwrap();
    let news = parse_files(&args.new, args.blacklist_file).unwrap();
    let files = Set::from_iter(olds.keys().cloned().chain(news.keys().cloned()));
    let mut diff = TotalDiff::new();

    // per file
    for file in files {
        diff.insert(file.clone(), Map::new());

        if !news.contains_key(&file) {
            debug!("{} got deleted", file);
            for extrinsic in olds.get(&file).unwrap() {
                diff.get_mut(&file)
                    .unwrap()
                    .insert(extrinsic.0.clone(), ExtrinsicDiff::Removed);
            }
            continue;
        } else if !olds.contains_key(&file) {
            debug!("{} got added", file);
            for extrinsic in news.get(&file).unwrap() {
                diff.get_mut(&file)
                    .unwrap()
                    .insert(extrinsic.0.clone(), ExtrinsicDiff::Added);
            }
            continue;
        }

        let olds = &olds[&file];
        let news = &news[&file];
        let extrinsics = Set::from_iter(olds.keys().cloned().chain(news.keys().cloned()));
        // per extrinsic
        for extrinsic in extrinsics {
            if !news.contains_key(&extrinsic) {
                debug!("{} got deleted", extrinsic);
                diff.get_mut(&file)
                    .unwrap()
                    .insert(extrinsic.clone(), ExtrinsicDiff::Removed);
                continue;
            } else if !olds.contains_key(&extrinsic) {
                debug!("{} got added", extrinsic);
                diff.get_mut(&file)
                    .unwrap()
                    .insert(extrinsic.clone(), ExtrinsicDiff::Added);
                continue;
            }

            let old_w = olds[&extrinsic];
            let new_w = news[&extrinsic];
            let p = percent(old_w, new_w);

            if p == 0.0 {
                diff.get_mut(&file)
                    .unwrap()
                    .insert(extrinsic.clone(), ExtrinsicDiff::Same);
            } else {
                diff.get_mut(&file)
                    .unwrap()
                    .insert(extrinsic.clone(), ExtrinsicDiff::Change(old_w, new_w, p));
            }
        }
    }

    let mut changed = Vec::new();
    for (file, diff) in diff.iter() {
        for (extrinsic, diff) in diff {
            match diff {
                ExtrinsicDiff::Change(old, new, p) => {
                    changed.push(((file, extrinsic), old, new, p))
                }
                _ => {}
            }
        }
    }
    changed.sort_by(|a, b| a.3.abs().partial_cmp(&b.3.abs()).unwrap());

    for ((file, extrinsic), old, new, p) in changed.iter().cloned() {
        if *p < -args.threshold {
            info!(
                "{}::{} {} -> {} ns ({} %)",
                file,
                extrinsic,
                old,
                new,
                color_percent(*p)
            );
        }
    }
    for ((file, extrinsic), old, new, p) in changed.iter().cloned() {
        if *p > args.threshold {
            info!(
                "{}::{} {} -> {} ns ({} %)",
                file,
                extrinsic,
                old,
                new,
                color_percent(*p)
            );
        }
    }
}

fn percent(old: WeightNs, new: WeightNs) -> f64 {
    if old == 0 && new != 0 {
        return 100.0;
    } else if old != 0 && new == 0 {
        return -100.0;
    } else if old == 0 && new == 0 {
        return 0.0;
    } else {
        100.0 * (new as f64 / old as f64) - 100.0
    }
}

fn color_percent(p: f64) -> String {
    use ansi_term::Colour;

    if p < 0.0 {
        Colour::Green.paint(format!("-{:.2?}", p.abs()))
    } else if p > 0.0 {
        Colour::Red.paint(format!("+{:.2?}", p.abs()))
    } else {
        // 0 or NaN
        Colour::White.paint(format!("{:.0?}", p))
    }
    .to_string()
}

// retusn file -> extrinsics
fn parse_files(
    paths: &Vec<PathBuf>,
    blacklist: Vec<String>,
) -> Result<Map<String, Map<String, WeightNs>>, String> {
    let mut map = Map::new();
    'outer: for path in paths {
        for skip in &blacklist {
            if path.to_string_lossy().to_string().ends_with(skip) {
                continue 'outer;
            }
        }
        map.insert(file_of(path), parse_file(path)?);
    }
    Ok(map)
}

/// Strips the path and only returns the file name.
fn file_of(path: &PathBuf) -> String {
    path.file_name().unwrap().to_str().unwrap().to_string()
}

fn parse_file(path: &PathBuf) -> Result<Map<String, WeightNs>, String> {
    let mut file = ::std::fs::File::open(&path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let ast = syn::parse_file(&content).unwrap();
    for item in ast.items {
        if let Some(weights) = handle_item(&item) {
            return Ok(weights);
        }
    }
    Err(format!(
        "Could not find weight implementation in the passed file: {}\n\
    Ensure that you are using the template from the Substrate .maintain folder.",
        path.display()
    )
    .into())
}

fn handle_item(item: &Item) -> Option<Map<String, WeightNs>> {
    match item {
        Item::Impl(imp) => {
            match imp.self_ty.as_ref() {
                Type::Tuple(t) if t.elems.is_empty() => return None,
                _ => {}
            }
            let mut weights = Map::new();
            for f in &imp.items {
                match f {
                    ImplItem::Method(m) => {
                        let (name, weight) = handle_method(m);
                        weights.insert(name, weight);
                    }
                    _ => {}
                }
            }
            Some(weights)
        }
        _ => None,
    }
}

fn handle_method(m: &ImplItemMethod) -> (String, WeightNs) {
    let name = m.sig.ident.to_string();
    assert_eq!(
        m.block.stmts.len(),
        1,
        "There must be only one statement per weight function"
    );
    let stmt = m.block.stmts.first().unwrap();

    let weight = match stmt {
        Stmt::Expr(expr) => extract_base_weight(expr),
        _ => unreachable!("Expected expression"),
    };
    (name, weight)
}

/// Recursively descends until it finds the base weight integer.
fn extract_base_weight(expr: &Expr) -> WeightNs {
    match expr {
        Expr::Paren(expr) => extract_base_weight(&expr.expr),
        Expr::Cast(cast) => extract_base_weight(&cast.expr),
        Expr::MethodCall(call) => extract_base_weight(&call.receiver),
        Expr::Lit(lit) => lit_to_weight(&lit.lit),
        _ => unreachable!(),
    }
}

fn lit_to_weight(lit: &Lit) -> WeightNs {
    match lit {
        Lit::Int(i) => i
            .base10_digits()
            .parse::<u64>()
            .unwrap()
            .checked_div(1000)
            .unwrap(),
        _ => unreachable!(),
    }
}
