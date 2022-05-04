use clap::Args;
use log::*;
use std::collections::BTreeMap as Map;
use std::collections::BTreeSet as Set;
use std::path::Path;
use std::path::PathBuf;
use syn::{Expr, ImplItem, ImplItemMethod, Item, Lit, Stmt, Type};

pub mod parse;
#[cfg(test)]
mod test;

use parse::parse_files;

// 1000 weight
type WeightNs = u64;
type Percent = f64;

pub enum ExtrinsicChange {
    Same,
    Added,
    Removed,
    Change(WeightNs, WeightNs, Percent),
}

pub struct ExtrinsicDiff {
    pub name: String,
    pub file: String,
    pub old: WeightNs,
    pub new: WeightNs,
    pub change: Percent,
}

/// Parameters for modifying the benchmark behaviour.
#[derive(Debug, Default, Clone, PartialEq, Args)]
pub struct CompareParams {
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

// File -> Extrinsic -> Diff
pub type TotalDiff = Map<String, Map<String, ExtrinsicChange>>;

pub fn compare_files(params: &CompareParams) -> TotalDiff {
    let olds = parse_files(&params.old, &params.blacklist_file).unwrap();
    let news = parse_files(&params.new, &params.blacklist_file).unwrap();
    let files = Set::from_iter(olds.keys().cloned().chain(news.keys().cloned()));
    let mut diff = TotalDiff::new();

    // per file
    for file in files {
        diff.insert(file.clone(), Map::new());

        if !news.contains_key(&file) {
            warn!("{} got deleted", file);
            for extrinsic in olds.get(&file).unwrap() {
                diff.get_mut(&file)
                    .unwrap()
                    .insert(extrinsic.0.clone(), ExtrinsicChange::Removed);
            }
            continue;
        } else if !olds.contains_key(&file) {
            debug!("{} got added", file);
            for extrinsic in news.get(&file).unwrap() {
                diff.get_mut(&file)
                    .unwrap()
                    .insert(extrinsic.0.clone(), ExtrinsicChange::Added);
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
                    .insert(extrinsic.clone(), ExtrinsicChange::Removed);
                continue;
            } else if !olds.contains_key(&extrinsic) {
                debug!("{} got added", extrinsic);
                diff.get_mut(&file)
                    .unwrap()
                    .insert(extrinsic.clone(), ExtrinsicChange::Added);
                continue;
            }

            let old_w = olds[&extrinsic];
            let new_w = news[&extrinsic];
            let p = percent(old_w, new_w);

            if p == 0.0 {
                diff.get_mut(&file)
                    .unwrap()
                    .insert(extrinsic.clone(), ExtrinsicChange::Same);
            } else {
                diff.get_mut(&file)
                    .unwrap()
                    .insert(extrinsic.clone(), ExtrinsicChange::Change(old_w, new_w, p));
            }
        }
    }

    diff
}

pub fn extract_changes(params: &CompareParams, diff: TotalDiff) -> Vec<ExtrinsicDiff> {
    let mut changed = Vec::new();
    for (file, diff) in diff.iter() {
        for (extrinsic, diff) in diff {
            if let ExtrinsicChange::Change(old, new, p) = diff {
                if p.abs() >= params.threshold {
                    changed.push(ExtrinsicDiff {
                        name: extrinsic.clone(),
                        file: file.clone(),
                        old: *old,
                        new: *new,
                        change: *p,
                    });
                }
            }
        }
    }
    changed.sort_by(|b, a| a.change.partial_cmp(&b.change).unwrap());
    changed
}

pub fn fmt_changes(changes: &[ExtrinsicDiff]) -> Vec<String> {
    let mut out = Vec::new();
    for diff in changes {
        out.push(format!(
            "{:>40}::{:<40} {:>12} -> {:<12} ns ({:<12} %)",
            diff.file,
            diff.name,
            diff.old,
            diff.new,
            color_percent(diff.change),
        ));
    }
    out
}

fn percent(old: WeightNs, new: WeightNs) -> f64 {
    if old == 0 && new != 0 {
        100.0
    } else if old != 0 && new == 0 {
        -100.0
    } else if old == 0 && new == 0 {
        0.0
    } else {
        100.0 * (new as f64 / old as f64) - 100.0
    }
}

fn color_percent(p: f64) -> String {
    use ansi_term::Colour;

    let s = format!("{:+5.2}", p);
    match p {
        x if x < 0.0 => Colour::Green.paint(s),
        x if x > 0.0 => Colour::Red.paint(s),
        _ => Colour::White.paint(s),
    }
    .to_string()
}
