//! Parse and compare weight Substrate weight files.

use clap::Args;
use git2::*;
use git_version::git_version;
use lazy_static::lazy_static;
use log::*;
use prettytable::{cell, row, table};
use std::{
	collections::{BTreeMap as Map, BTreeSet as Set},
	path::{Path, PathBuf},
};
use syn::{Expr, Item, Type};

pub mod parse;
pub mod scope;
pub mod term;
pub mod testing;

#[cfg(test)]
mod test;

use parse::pallet::*;
use term::{multivariadic_eval, Term};

lazy_static! {
	/// Version of the library. Example: `swc 0.2.0+78a04b2-modified`.
	pub static ref VERSION: String = format!("{}+{}", env!("CARGO_PKG_VERSION"), git_version!());
}

type Percent = f64;

pub enum ExtrinsicChange {
	Same,
	Added,
	Removed,
	Change(u128, u128, Percent),
}

pub struct ExtrinsicDiff {
	pub name: String,
	pub file: String,
	pub old: u128,
	pub new: u128,
	pub change: Percent,
}

/// Parameters for modifying the benchmark behaviour.
#[derive(Debug, Default, Clone, PartialEq, Args)]
pub struct CompareParams {
	#[clap(long, value_name = "PERCENT", default_value = "5")]
	pub threshold: Percent,
}

// File -> Extrinsic -> Diff
pub type TotalDiff = Map<String, Map<String, ExtrinsicChange>>;

pub fn compare_commits(
	repo: &Path,
	old: &str,
	new: &str,
	thresh: Percent,
) -> Result<Vec<ExtrinsicDiff>, String> {
	// Parse the old files.
	if let Err(err) = checkout(repo, old) {
		return Err(format!("{:?}", err))
	}
	let paths = list_files(format!("{}/runtime/polkadot/src/weights/*.rs", repo.display()));
	let olds = parse_files(&paths).unwrap();

	// Parse the new files.
	if let Err(err) = checkout(repo, new) {
		return Err(format!("{:?}", err))
	}
	let news = parse_files(&paths).unwrap();
	let diff = compare_files(olds, news);

	Ok(extract_changes(diff, thresh))
}

/// Check out a repo to a given *commit*, *branch* or *tag*.
pub fn checkout(path: &Path, refname: &str) -> Result<(), git2::Error> {
	let repo = Repository::open(path)?;

	let (object, reference) = repo.revparse_ext(refname)?;
	repo.checkout_tree(&object, None)?;

	match reference {
		// gref is an actual reference like branches or tags
		Some(gref) => repo.set_head(gref.name().unwrap()),
		// this is a commit, not a reference
		None => repo.set_head_detached(object.id()),
	}
}

fn list_files(regex: String) -> Vec<PathBuf> {
	let files = glob::glob(&regex).unwrap();
	files.map(|f| f.unwrap()).filter(|f| !f.ends_with("mod.rs")).collect()
}

pub fn compare_extrinsic(old: &Term, new: &Term) -> ExtrinsicChange {
	// Default Substrate scope to KISS.
	let scope = scope::BasicScope::empty().with_storage_weights(val!(8_000), val!(50_000));
	// Use 100 until <https://github.com/paritytech/substrate/issues/11397> is done.
	let max = 100;

	let worst_old = multivariadic_eval(old, scope.clone(), max);
	let worst_new = multivariadic_eval(new, scope.clone(), max);
	let p = percent(worst_old, worst_new);

	if p == 0.0 {
		ExtrinsicChange::Same
	} else {
		ExtrinsicChange::Change(worst_old, worst_new, p)
	}
}

pub fn compare_files(olds: ParsedFiles, news: ParsedFiles) -> TotalDiff {
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
			continue
		} else if !olds.contains_key(&file) {
			debug!("{} got added", file);
			for extrinsic in news.get(&file).unwrap() {
				diff.get_mut(&file).unwrap().insert(extrinsic.0.clone(), ExtrinsicChange::Added);
			}
			continue
		}

		let olds = &olds[&file];
		let news = &news[&file];
		let extrinsics = Set::from_iter(olds.keys().cloned().chain(news.keys().cloned()));
		// per extrinsic
		for extrinsic in extrinsics {
			if !news.contains_key(&extrinsic) {
				debug!("{} got deleted", extrinsic);
				diff.get_mut(&file).unwrap().insert(extrinsic.clone(), ExtrinsicChange::Removed);
				continue
			} else if !olds.contains_key(&extrinsic) {
				debug!("{} got added", extrinsic);
				diff.get_mut(&file).unwrap().insert(extrinsic.clone(), ExtrinsicChange::Added);
				continue
			}

			let old_w = &olds[&extrinsic];
			let new_w = &news[&extrinsic];
			let change = compare_extrinsic(old_w, new_w);

			diff.get_mut(&file).unwrap().insert(extrinsic.clone(), change);
		}
	}

	diff
}

pub fn extract_changes(diff: TotalDiff, threshold: Percent) -> Vec<ExtrinsicDiff> {
	let mut changed = Vec::new();
	for (file, diff) in diff.iter() {
		for (extrinsic, diff) in diff {
			if let ExtrinsicChange::Change(old, new, p) = diff {
				if p.abs() >= threshold {
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

pub fn fmt_changes(changes: &[ExtrinsicDiff]) -> String {
	let mut table = table!(["Pallet", "Extrinsic", "Old", "New", "Change [%]"]);

	for diff in changes {
		table.add_row(row![
			diff.file,
			diff.name,
			fmt_value(diff.old),
			fmt_value(diff.new),
			color_percent(diff.change),
		]);
	}
	table.to_string()
}

pub fn percent(old: u128, new: u128) -> f64 {
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

pub fn color_percent(p: f64) -> String {
	use ansi_term::Colour;

	let s = format!("{:+5.2}", p);
	match p {
		x if x < 0.0 => Colour::Green.paint(s),
		x if x > 0.0 => Colour::Red.paint(s),
		_ => Colour::White.paint(s),
	}
	.to_string()
}

/// Hardcoded to avoid frame_support as dependency…
pub const WEIGHT_PER_PICOS: u128 = 1;
pub const WEIGHT_PER_NANOS: u128 = 1_000 * WEIGHT_PER_PICOS;
pub const WEIGHT_PER_MICROS: u128 = 1_000 * WEIGHT_PER_NANOS;
pub const WEIGHT_PER_MILLIS: u128 = 1_000 * WEIGHT_PER_MICROS;
pub const WEIGHT_PER_SECS: u128 = 1_000 * WEIGHT_PER_MILLIS;

pub fn fmt_value(w: u128) -> String {
	if w >= WEIGHT_PER_SECS {
		format!("{:.2}s", w as f64 / WEIGHT_PER_SECS as f64)
	} else if w >= WEIGHT_PER_MILLIS {
		format!("{:.2}ms", w as f64 / WEIGHT_PER_MILLIS as f64)
	} else if w >= WEIGHT_PER_MICROS {
		format!("{:.2}μs", w as f64 / WEIGHT_PER_MICROS as f64)
	} else if w >= WEIGHT_PER_NANOS {
		format!("{:.2}ns", w as f64 / WEIGHT_PER_NANOS as f64)
	} else {
		// Best effort approach: Just assume its actually not a weight
		// since the substrate benchmarking resolution is [`WEIGHT_PER_NANOS`].
		// Works fine in 99% of cases.
		format!("{}", w)
	}
}

/// Put an underscore after each third digit. 1000 -> 1_000
pub fn fmt_with_underscore(val: u128) -> String {
	let mut res = String::new();
	let s = val.to_string();

	for (i, char) in s.chars().rev().enumerate() {
		if i % 3 == 0 && i != 0 {
			res.insert(0, '_');
		}
		res.insert(0, char);
	}
	res
}
