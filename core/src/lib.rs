//! Parse and compare weight Substrate weight files.

use clap::Args;
use git2::*;
use git_version::git_version;
use lazy_static::lazy_static;

use regex::Regex;
use std::{
	cmp::Ordering,
	path::{Path, PathBuf},
};
use syn::{Expr, Item, Type};

pub mod parse;
pub mod scope;
pub mod term;
pub mod testing;

#[cfg(test)]
mod test;

use parse::pallet::{parse_files_in_repo, try_parse_files_in_repo, Extrinsic};
use scope::Scope;
use term::{multivariadic_eval, Term};

lazy_static! {
	/// Version of the library. Example: `swc 0.2.0+78a04b2-dirty`.
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

// TODO rename
#[derive(Debug, serde::Deserialize, clap::ArgEnum, Clone, Eq, Ord, PartialEq, PartialOrd, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum RelativeChange {
	Unchanged,
	Added,
	Removed,
	Changed,
}

/// Parameters for modifying the benchmark behaviour.
#[derive(Debug, Clone, PartialEq, Args)]
pub struct CompareParams {
	#[clap(
		long,
		short,
		value_name = "METHOD",
		ignore_case = true,
		possible_values = CompareMethod::variants(),
	)]
	pub method: CompareMethod,

	#[clap(
		long,
		short,
		value_name = "UNIT",
		ignore_case = true,
		default_value = "weight",
		possible_values = Unit::variants(),
	)]
	pub unit: Unit,

	#[clap(long)]
	pub ignore_errors: bool,
}

#[derive(Debug, Clone, PartialEq, Args)]
pub struct FilterParams {
	/// Minimal magnitude of a relative change to be relevant.
	#[clap(long, value_name = "PERCENT", default_value = "5")]
	pub threshold: Percent,

	/// Only include a subset of change-types.
	#[clap(long, ignore_case = true, multiple_values = true, value_name = "CHANGE-TYPE")]
	pub change: Option<Vec<RelativeChange>>,

	#[clap(long, ignore_case = true, value_name = "REGEX")]
	pub extrinsic: Option<String>,
}

pub fn compare_commits(
	repo: &Path,
	old: &str,
	new: &str,
	params: &CompareParams,
	path_pattern: &str,
	max_files: usize,
) -> Result<TotalDiff, String> {
	if path_pattern.contains("..") {
		return Err("Path pattern cannot contain '..'".to_string())
	}
	// Parse the old files.
	if let Err(err) = checkout(repo, old) {
		return Err(format!("{:?}", err))
	}
	let pattern = format!("{}/{}", repo.display(), path_pattern);
	let paths = list_files(pattern.clone(), max_files)?;
	// Ignore any parsing errors.
	let olds = if params.ignore_errors {
		try_parse_files_in_repo(repo, &paths)
	} else {
		// TODO use option for repo
		parse_files_in_repo(repo, &paths)?
	};

	// Parse the new files.
	if let Err(err) = checkout(repo, new) {
		return Err(format!("{:?}", err))
	}
	let paths = list_files(pattern, max_files)?;
	// Ignore any parsing errors.
	let news = if params.ignore_errors {
		try_parse_files_in_repo(repo, &paths)
	} else {
		parse_files_in_repo(repo, &paths)?
	};

	let diff = compare_files(olds, news, params.method);
	Ok(diff)
}

/// Check out a repo to a given *commit*, *branch* or *tag*.
pub fn checkout(path: &Path, refname: &str) -> Result<(), git2::Error> {
	let repo = Repository::open(path)?;

	let (object, reference) = repo.revparse_ext(refname)?;
	repo.checkout_tree(&object, None)?;

	match reference {
		// gref is an actual reference like branches or tags
		Some(gref) => repo.set_head(gref.name().ok_or_else(|| git2::Error::from_str("No name"))?),
		// this is a commit, not a reference
		None => repo.set_head_detached(object.id()),
	}
}

fn list_files(regex: String, max_files: usize) -> Result<Vec<PathBuf>, String> {
	let files = glob::glob(&regex).map_err(|e| format!("Invalid path pattern: {:?}", e))?;
	let files = files
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| format!("Path pattern error: {:?}", e))?;
	let files: Vec<_> = files.iter().cloned().filter(|f| !f.ends_with("mod.rs")).collect();
	if files.len() > max_files {
		return Err(format!("Found too many files. Found: {}, Max: {}", files.len(), max_files))
	} else {
		Ok(files)
	}
}

#[derive(serde::Deserialize, clap::ArgEnum, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum CompareMethod {
	/// The constant base weight of the extrinsic.
	Base,
	/// The worst case weight by setting all variables to 100.
	///
	/// Assumes
	Worst,
}

#[derive(serde::Deserialize, clap::ArgEnum, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Unit {
	Weight,
	Time,
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

impl std::str::FromStr for Unit {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, String> {
		match s {
			"weight" => Ok(Self::Weight),
			"time" => Ok(Self::Time),
			_ => Err(format!("Unknown method: {}", s)),
		}
	}
}

impl Unit {
	pub fn variants() -> Vec<&'static str> {
		vec!["weight", "time"]
	}
}

impl FilterParams {
	pub fn included(&self, change: &RelativeChange) -> bool {
		self.change.as_ref().map_or(true, |s| s.contains(change))
	}
}

impl std::str::FromStr for RelativeChange {
	type Err = String;
	// TODO try clap ArgEnum
	fn from_str(s: &str) -> Result<Self, String> {
		match s {
			"unchanged" => Ok(Self::Unchanged),
			"changed" => Ok(Self::Changed),
			"added" => Ok(Self::Added),
			"removed" => Ok(Self::Removed),
			_ => Err(format!("Unknown change: {}", s)),
		}
	}
}

impl RelativeChange {
	pub fn variants() -> Vec<&'static str> {
		vec!["unchanged", "changed", "added", "removed"]
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
	let mut new_scope = scope;
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

	diff
}

pub fn sort_changes(diff: &mut TotalDiff) {
	diff.sort_by(|a, b| {
		let ord = a.change.change.cmp(&b.change.change).reverse();
		if ord == Ordering::Equal {
			if a.change.percent > b.change.percent {
				Ordering::Greater
			} else if a.change.percent == b.change.percent {
				Ordering::Equal
			} else {
				Ordering::Less
			}
		} else {
			ord
		}
	});
}

pub fn filter_changes(diff: TotalDiff, params: &FilterParams) -> TotalDiff {
	// Parse the extrinsic regex
	let regex = params.extrinsic.as_ref().map(|s| Regex::new(s).unwrap());
	diff.iter()
		.filter(|extrinsic| regex.as_ref().map_or(true, |r| r.is_match(&extrinsic.name)))
		.filter(|extrinsic| params.included(&extrinsic.change.change))
		.filter(|extrinsic| match extrinsic.change.change {
			RelativeChange::Changed if extrinsic.change.percent.abs() < params.threshold => false,
			RelativeChange::Unchanged if params.threshold >= 0.001 => false,

			_ => true,
		})
		.cloned()
		.collect()
}

impl RelativeChange {
	pub fn new(old: Option<u128>, new: Option<u128>) -> RelativeChange {
		match (old, new) {
			(old, new) if old == new => RelativeChange::Unchanged,
			(Some(_), Some(_)) => RelativeChange::Changed,
			(None, Some(_)) => RelativeChange::Added,
			(Some(_), None) => RelativeChange::Removed,
			(None, None) => unreachable!("Either old or new must be set"),
		}
	}
}

pub fn percent(old: u128, new: u128) -> Percent {
	100.0 * (new as f64 / old as f64) - 100.0
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

/// Formats pico seconds.
pub fn fmt_time(t: u128) -> String {
	if t >= 1_000_000_000_000 {
		format!("{:.2}s", t as f64 / 1_000_000_000_000f64)
	} else if t >= 1_000_000_000 {
		format!("{:.2}ms", t as f64 / 1_000_000_000f64)
	} else if t >= 1_000_000 {
		format!("{:.2}us", t as f64 / 1_000_000f64)
	} else if t >= 1_000 {
		format!("{:.2}ns", t as f64 / 1_000f64)
	} else {
		format!("{:.2}ps", t)
	}
}

impl Unit {
	pub fn fmt_value(&self, v: u128) -> String {
		match self {
			Unit::Time => fmt_time(v),
			Unit::Weight => fmt_weight(v),
		}
	}
}
