//! Parse and compare weight Substrate weight files.

use clap::Args;
use git2::*;
use git_version::git_version;
use lazy_static::lazy_static;

use prettytable::{cell, row, table};
use std::path::{Path, PathBuf};
use syn::{Expr, Item, Type};

pub mod parse;
pub mod scope;
pub mod term;
pub mod testing;

#[cfg(test)]
mod test;

use parse::pallet::{parse_files, try_parse_files, Extrinsic};
use scope::Scope;
use term::{multivariadic_eval, Term};

lazy_static! {
	/// Version of the library. Example: `swc 0.2.0+78a04b2-modified`.
	pub static ref VERSION: String = format!("{}+{}", env!("CARGO_PKG_VERSION"), git_version!(args = ["--dirty", "--always"], fallback = "unknown"));
	pub static ref VERSION_DIRTY: bool = {
		VERSION.clone().contains("dirty")
	};
}

pub type PalletName = String;
pub type ExtrinsicName = String;
pub type TotalDiff = Vec<ExtrinsicDiff>;

pub type Percent = f64;
pub const WEIGHT_PER_NANOS: u128 = 1_000;

#[derive(Clone)]
pub struct ExtrinsicDiff {
	pub name: ExtrinsicName,
	pub file: String,

	pub change: TermChange,
}

#[derive(Clone)]
// Uses options since extrinsics can be added or removed and any time.
pub struct TermChange {
	pub old: Option<Term>,
	pub old_v: Option<u128>,

	pub new: Option<Term>,
	pub new_v: Option<u128>,

	pub scope: Scope,
	pub percent: Percent,
	pub change: RelativeChange,
	pub method: CompareMethod,
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Copy)]
pub enum RelativeChange {
	Unchanged,
	Added,
	Removed,
	Change,
}

/// Parameters for modifying the benchmark behaviour.
#[derive(Debug, Clone, PartialEq, Args)]
pub struct CompareParams {
	#[clap(long, value_name = "PERCENT", default_value = "5")]
	pub threshold: Percent,

	#[clap(
		long,
		short,
		value_name = "METHOD",
		ignore_case = true,
		possible_values = CompareMethod::variants(),
	)]
	pub method: CompareMethod,

	#[clap(long)]
	pub path_pattern: String,

	#[clap(long)]
	pub ignore_errors: bool,
}

pub fn compare_commits(
	repo: &Path,
	old: &str,
	new: &str,
	thresh: Percent,
	method: CompareMethod,
	path_pattern: &str,
	ignore_errors: bool,
	max_files: usize,
) -> Result<TotalDiff, String> {
	if path_pattern.contains("..") {
		return Err(format!("Path pattern cannot contain '..'"))
	}
	// Parse the old files.
	if let Err(err) = checkout(repo, old) {
		return Err(format!("{:?}", err))
	}
	let pattern = format!("{}/{}", repo.display(), path_pattern);
	let paths = list_files(pattern.clone(), max_files)?;
	// Ignore any parsing errors.
	let olds =
		if ignore_errors { try_parse_files(repo, &paths) } else { parse_files(repo, &paths)? };

	// Parse the new files.
	if let Err(err) = checkout(repo, new) {
		return Err(format!("{:?}", err))
	}
	let paths = list_files(pattern, max_files)?;
	// Ignore any parsing errors.
	let news =
		if ignore_errors { try_parse_files(repo, &paths) } else { parse_files(repo, &paths)? };

	let diff = compare_files(olds, news, thresh, method);
	Ok(filter_changes(diff, thresh))
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

fn list_files(regex: String, max_files: usize) -> Result<Vec<PathBuf>, String> {
	let files = glob::glob(&regex).unwrap();
	let files: Vec<_> = files.map(|f| f.unwrap()).filter(|f| !f.ends_with("mod.rs")).collect();
	if files.len() > max_files {
		return Err(
			format!("Too many files found. Found: {}, Max: {}", files.len(), max_files).into()
		)
	} else {
		Ok(files)
	}
}

#[derive(serde::Deserialize, clap::ArgEnum, PartialEq, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CompareMethod {
	/// The constant base weight of the extrinsic.
	Base,
	/// The worst case weight by setting all variables to 100.
	///
	/// Assumes
	Worst,
}

impl std::str::FromStr for CompareMethod {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, String> {
		match s {
			"base" => Ok(CompareMethod::Base),
			"worst" => Ok(CompareMethod::Worst),
			_ => Err(format!("Unknown method: {}", s)),
		}
	}
}

impl CompareMethod {
	pub fn variants() -> Vec<&'static str> {
		vec!["base", "worst"]
	}
}

pub fn compare_terms(old: Option<&Term>, new: Option<&Term>, method: CompareMethod) -> TermChange {
	let mut max = 0;
	// Default substrate storage weights
	let scope = scope::Scope::empty().with_storage_weights(val!(25_000), val!(100_000));

	if method == CompareMethod::Worst {
		// Use 100 until <https://github.com/paritytech/substrate/issues/11397> is done.
		max = 100;
	}

	let mut old_scope = scope.clone();
	let old_v = old.map(|t| multivariadic_eval(t, &mut old_scope, max));
	let mut new_scope = scope.clone();
	let new_v = new.map(|t| multivariadic_eval(t, &mut new_scope, max));
	let change = RelativeChange::new(old_v, new_v);
	let p = percent(old_v.unwrap_or_default(), new_v.unwrap_or_default());

	let merged = old_scope.merge(new_scope);
	TermChange {
		old: old.cloned(),
		old_v,
		new: new.cloned(),
		new_v,
		change,
		percent: p,
		method,
		scope: merged,
	}
}

pub fn compare_files(
	olds: Vec<Extrinsic>,
	news: Vec<Extrinsic>,
	thresh: Percent,
	method: CompareMethod,
) -> TotalDiff {
	let mut diff = TotalDiff::new();
	let old_names = olds.iter().cloned().map(|e| (e.pallet, e.name));
	let new_names = news.iter().cloned().map(|e| (e.pallet, e.name));
	let names = old_names.chain(new_names).collect::<std::collections::BTreeSet<_>>();

	for (pallet, extrinsic) in names {
		let new = news
			.iter()
			.find(|&n| n.name == extrinsic && n.pallet == pallet)
			.map(|e| &e.term);
		let old = olds
			.iter()
			.find(|&n| n.name == extrinsic && n.pallet == pallet)
			.map(|e| &e.term);

		let change = compare_terms(old, new, method);
		let change = ExtrinsicDiff { name: extrinsic.clone(), file: pallet.clone(), change };

		diff.push(change);
	}

	filter_changes(diff, thresh)
}

pub fn filter_changes(diff: TotalDiff, threshold: Percent) -> TotalDiff {
	diff.iter()
		.filter(|extrinsic| match extrinsic.change.change {
			RelativeChange::Change if extrinsic.change.percent.abs() < threshold => false,
			RelativeChange::Unchanged if threshold >= 0.001 => false,

			_ => true,
		})
		.cloned()
		.collect()
}

pub fn fmt_changes(changes: &TotalDiff) -> String {
	// Collect the extrinsics by category into a vector each.
	let mut changed = Vec::new();
	let mut unchanged = Vec::new();
	let mut added = Vec::new();
	let mut removed = Vec::new();

	for extrinsic in changes.iter() {
		match extrinsic.change.change {
			RelativeChange::Change => changed.push(extrinsic),
			RelativeChange::Unchanged => unchanged.push(extrinsic),
			RelativeChange::Added => added.push(extrinsic),
			RelativeChange::Removed => removed.push(extrinsic),
		}
	}

	let mut table = table!(["Pallet", "Extrinsic", "Old", "New", "Change [%]"]);

	for diff in changed {
		table.add_row(row![
			diff.file,
			diff.name,
			diff.change.old_v.map(fmt_weight).unwrap_or_default(),
			diff.change.new_v.map(fmt_weight).unwrap_or_default(),
			color_percent(diff.change.percent, &diff.change.change),
		]);
	}
	table.to_string()
}

impl RelativeChange {
	pub fn new(old: Option<u128>, new: Option<u128>) -> RelativeChange {
		match (old, new) {
			(old, new) if old == new => RelativeChange::Unchanged,
			(Some(_), Some(_)) => RelativeChange::Change,
			(None, Some(_)) => RelativeChange::Added,
			(Some(_), None) => RelativeChange::Removed,
			(None, None) => unreachable!("Either old or new must be set"),
		}
	}
}

pub fn percent(old: u128, new: u128) -> Percent {
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

// TODO remove
pub fn color_percent(p: Percent, change: &RelativeChange) -> String {
	use ansi_term::Colour;

	match change {
		RelativeChange::Unchanged => "0.00% (No change)".to_string(),
		RelativeChange::Added => Colour::Red.paint("100.00% (Added)").to_string(),
		RelativeChange::Removed => Colour::Green.paint("-100.00% (Removed)").to_string(),
		RelativeChange::Change => {
			let s = format!("{:+5.2}", p);
			match p {
				x if x < 0.0 => Colour::Green.paint(s),
				x if x > 0.0 => Colour::Red.paint(s),
				_ => Colour::White.paint(s),
			}
			.to_string()
		},
	}
}

pub fn fmt_weight(w: u128) -> String {
	if w >= 1_000_000_000_000 {
		format!("{:.2}T", w as f64 / 1_000_000_000_000f64)
	} else if w >= 1_000_000_000 {
		format!("{:.2}G", w as f64 / 1_000_000_000f64)
	} else if w >= 1_000_000 {
		format!("{:.2}M", w as f64 / 1_000_000f64)
	} else if w >= 1_000 {
		format!("{:.2}K", w as f64 / 1_000f64)
	} else {
		w.to_string()
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
