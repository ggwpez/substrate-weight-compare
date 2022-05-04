use log::debug;
use std::io::Read;
use std::path::PathBuf;

use crate::*;

pub type PalletName = String;
pub type ExtrinsicName = String;

/// Maps an Extrinsic in the form of (PalletName, ExtrinsicName) to its Weight.
///
/// NOTE: Uses a 2D map for prefix matching.
pub type ParsedFiles = Map<PalletName, Map<ExtrinsicName, WeightNs>>;
pub type ParsedExtrinsic = Map<ExtrinsicName, WeightNs>;

const LOG: &str = "parser";

pub fn parse_files(paths: &Vec<PathBuf>, blacklist: &Vec<String>) -> Result<ParsedFiles, String> {
    let mut map = Map::new();
    'outer: for path in paths {
        for skip in blacklist {
            if path.to_string_lossy().to_string().ends_with(skip) {
                continue 'outer;
            }
        }
        map.insert(file_of(path), parse_file(path)?);
    }
    Ok(map)
}

/// Strips the path and only returns the file name.
fn file_of(path: &Path) -> String {
    path.file_name().unwrap().to_str().unwrap().to_string()
}

pub fn parse_file(path: &PathBuf) -> Result<ParsedExtrinsic, String> {
    debug!(target: LOG, "Entering file: {}", path.to_string_lossy());
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
    ))
}

fn handle_item(item: &Item) -> Option<Map<String, WeightNs>> {
    match item {
        Item::Impl(imp) => {
            match imp.self_ty.as_ref() {
                Type::Tuple(t) if t.elems.is_empty() => {
                    debug!(target: LOG, "Skipped fn: impl tuple type empty");
                    // The substrate template contains the weight info twice.
                    // By skipping the `impl ()` we ensure to parse it only once.
                    return None;
                }
                Type::Path(p) => {
                    if p.path.leading_colon.is_some() {
                        debug!(target: LOG, "Skipped fn: impl leading color");
                        return None;
                    }
                    if p.path.segments.len() != 1 {
                        debug!(target: LOG, "Skipped fn: impl path segment len");
                        return None;
                    }
                    if let Some(last) = p.path.segments.last() {
                        let name = last.ident.to_string();
                        if name != "WeightInfo" {
                            debug!(target: LOG, "Skipped fn: impl name last: {}", name);
                            return None;
                        }
                        debug!(target: LOG, "Using fn: impl name: {}", name);
                    } else {
                        debug!(target: LOG, "Skipped fn: impl name segments");
                        return None;
                    }
                }
                _ => {
                    debug!(target: LOG, "Skipped fn: impl type");
                    return None;
                }
            }
            let mut weights = Map::new();
            for f in &imp.items {
                if let ImplItem::Method(m) = f {
                    let (name, weight) = handle_method(m);
                    weights.insert(name, weight);
                }
            }
            Some(weights)
        }
        _ => None,
    }
}

fn handle_method(m: &ImplItemMethod) -> (String, WeightNs) {
    let name = m.sig.ident.to_string();
    debug!(target: LOG, "Enter function {}", name);
    assert_eq!(
        m.block.stmts.len(),
        1,
        "There must be only one statement per weight function: {}",
        name
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
