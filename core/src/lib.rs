//! Parse and compare weight Substrate weight files.

use clap::Args;
use fancy_regex::Regex;
use git_version::git_version;
use lazy_static::lazy_static;

use std::{
	cmp::Ordering,
	collections::HashSet,
	path::{Path, PathBuf},
	process::Command,
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
use term::Term;

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

	pub change: TermDiff,
}

#[derive(Clone)]
pub enum TermDiff {
	Changed(TermChange),
	/// There was an error while comparing the old to the new version.
	Failed(String),
}

impl ExtrinsicDiff {
	pub fn term(&self) -> Option<&TermChange> {
		match &self.change {
			TermDiff::Changed(change) => Some(change),
			TermDiff::Failed(_) => None,
		}
	}

	pub fn error(&self) -> Option<&String> {
		match &self.change {
			TermDiff::Changed(_) => None,
			TermDiff::Failed(err) => Some(err),
		}
	}
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

	/// Do a 'git pull' after checking out the refname.
	///
	/// This ensures that you get the newest commit on a branch.
	#[clap(long)]
	pub git_pull: bool,
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

	#[clap(long, alias("file"), ignore_case = true, value_name = "REGEX")]
	pub pallet: Option<String>,
}

pub fn compare_commits(
	repo: &Path,
	old: &str,
	new: &str,
	params: &CompareParams,
	filter: &FilterParams,
	path_pattern: &str,
	max_files: usize,
) -> Result<TotalDiff, Box<dyn std::error::Error>> {
	if path_pattern.contains("..") {
		return Err("Path pattern cannot contain '..'".into())
	}
	// Parse the old files.
	if let Err(err) = reset(repo, old) {
		return Err(format!("{:?}", err).into())
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
	if let Err(err) = reset(repo, new) {
		return Err(format!("{:?}", err).into())
	}
	let paths = list_files(pattern, max_files)?;
	// Ignore any parsing errors.
	let news = if params.ignore_errors {
		try_parse_files_in_repo(repo, &paths)
	} else {
		parse_files_in_repo(repo, &paths)?
	};

	compare_files(olds, news, params.method, filter)
}

pub fn reset(path: &Path, refname: &str) -> Result<(), String> {
	// fetch the single branch
	log::info!("Fetching branch {}", refname);
	if !is_commit(refname) {
		let output = Command::new("git")
			.arg("fetch")
			.arg("origin")
			.arg(refname)
			.current_dir(path)
			.output()
			.map_err(|e| format!("Failed to fetch branch: {:?}", e))?;
		if !output.status.success() {
			return Err(format!(
				"Failed to fetch branch: {}",
				String::from_utf8_lossy(&output.stderr)
			))
		}
	}
	// hard reset
	let output = if is_commit(refname) {
		log::info!("Resetting to branch {}", refname);
		Command::new("git")
			.arg("reset")
			.arg("--hard")
			.arg(refname)
			.current_dir(path)
			.output()
	} else {
		log::info!("Resetting to branch origin/{}", refname);
		Command::new("git")
			.arg("reset")
			.arg("--hard")
			.arg(format!("origin/{}", refname))
			.current_dir(path)
			.output()
	}
	.map_err(|e| format!("Failed to reset branch: {:?}", e))?;

	if !output.status.success() {
		return Err(format!("Failed to reset branch: {}", String::from_utf8_lossy(&output.stderr)))
	}
	Ok(())
}

/// Tries to guess whether a refname is a commit hash or not.
fn is_commit(refname: &str) -> bool {
	refname.chars().all(|c| c.is_ascii_hexdigit())
}

fn list_files(regex: String, max_files: usize) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
	let files = glob::glob(&regex).map_err(|e| format!("Invalid path pattern: {:?}", e))?;
	let files = files
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| format!("Path pattern error: {:?}", e))?;
	let files: Vec<_> = files.iter().cloned().filter(|f| !f.ends_with("mod.rs")).collect();
	if files.len() > max_files {
		return Err(
			format!("Found too many files. Found: {}, Max: {}", files.len(), max_files).into()
		)
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
	GuessWorst,

	ExactWorst,
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
			"guess-worst" => Ok(CompareMethod::GuessWorst),
			"exact-worst" => Ok(CompareMethod::ExactWorst),
			_ => Err(format!("Unknown method: {}", s)),
		}
	}
}

impl CompareMethod {
	pub fn variants() -> Vec<&'static str> {
		vec!["base", "guess-worst", "exact-worst"]
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

pub fn compare_extrinsics(
	old: Option<&Extrinsic>,
	new: Option<&Extrinsic>,
	method: CompareMethod,
) -> Result<TermChange, String> {
	let mut scope = scope::Scope::empty().with_storage_weights(val!(25_000_000), val!(100_000_000));
	extend_scoped_components(old, new, method, &mut scope)?;
	let name = old.map(|o| o.name.clone()).or_else(|| new.map(|n| n.name.clone())).unwrap();
	let pallet = old.map(|o| o.pallet.clone()).or_else(|| new.map(|n| n.pallet.clone())).unwrap();

	if !old.map_or(true, |e| e.term.free_vars(&scope).is_empty()) {
		panic!(
			"Free variable where there should be none: {}::{} {:?}",
			name,
			&pallet,
			old.unwrap().term.free_vars(&scope)
		);
	}
	assert!(new.map_or(true, |e| e.term.free_vars(&scope).is_empty()));

	compare_terms(old.map(|e| &e.term), new.map(|e| &e.term), method, &scope)
}

// TODO handle case that both have (different) ranges.
pub(crate) fn extend_scoped_components(
	a: Option<&Extrinsic>,
	b: Option<&Extrinsic>,
	method: CompareMethod,
	scope: &mut Scope,
) -> Result<(), String> {
	let free_a = a.map(|e| e.term.free_vars(scope)).unwrap_or_default();
	let free_b = b.map(|e| e.term.free_vars(scope)).unwrap_or_default();
	let frees = free_a.union(&free_b).cloned().collect::<HashSet<_>>();

	let ra = a.map(|ext| ext.clone().comp_ranges.unwrap_or_default());
	let rb = b.map(|ext| ext.clone().comp_ranges.unwrap_or_default());

	let (pallet, extrinsic) = a.or(b).map(|e| (e.pallet.clone(), e.name.clone())).unwrap();

	// Calculate a concrete value for each component.
	let values = frees
		.iter()
		.map(|component| {
			let v = match (
				ra.as_ref().and_then(|r| r.get(component)),
				rb.as_ref().and_then(|r| r.get(component)),
			) {
				// Only one extrinsic has a component range? Good
				(Some(r), None) | (None, Some(r)) => Ok(match method {
					CompareMethod::Base => r.min,
					CompareMethod::GuessWorst | CompareMethod::ExactWorst => r.max,
				}),
				// Both extrinsics have the same range? Good
				(Some(ra), Some(rb)) if ra == rb => Ok(match method {
					CompareMethod::Base => ra.min,
					CompareMethod::GuessWorst | CompareMethod::ExactWorst => ra.max,
				}),
				// Both extrinsics have different ranges? Bad, use the min/max
				(Some(ra), Some(rb)) => Ok(match method {
					CompareMethod::Base => ra.min.min(rb.min),
					CompareMethod::ExactWorst | CompareMethod::GuessWorst => ra.max.max(rb.max),
				}),
				// No ranges? Bad, just guess 100
				(None, None) => match method {
					CompareMethod::Base => Ok(0),
					CompareMethod::GuessWorst => Ok(100),
					CompareMethod::ExactWorst => Err(format!(
						"No range for component {} of call {}::{} - use GuessWorst instead!",
						component, pallet, extrinsic
					)),
				},
			};
			(component, v)
		})
		.collect::<Vec<_>>();

	for (component, value) in values {
		scope.put_var(component, val!(value?));
	}

	Ok(())
}

pub fn compare_terms(
	old: Option<&Term>,
	new: Option<&Term>,
	method: CompareMethod,
	scope: &Scope,
) -> Result<TermChange, String> {
	let old_v = old.map(|t| t.eval(scope)).transpose()?;
	let new_v = new.map(|t| t.eval(scope)).transpose()?;
	let change = RelativeChange::new(old_v, new_v);
	let p = percent(old_v.unwrap_or_default(), new_v.unwrap_or_default());

	Ok(TermChange {
		old: old.cloned(),
		old_v,
		new: new.cloned(),
		new_v,
		change,
		percent: p,
		method,
		scope: scope.clone(),
	})
}

pub fn compare_files(
	olds: Vec<Extrinsic>,
	news: Vec<Extrinsic>,
	method: CompareMethod,
	filter: &FilterParams,
) -> Result<TotalDiff, Box<dyn std::error::Error>> {
	let ext_regex = filter.extrinsic.as_ref().map(|s| Regex::new(s)).transpose()?;
	let pallet_regex = filter.pallet.as_ref().map(|s| Regex::new(s)).transpose()?;

	let mut diff = TotalDiff::new();
	let old_names = olds.iter().cloned().map(|e| (e.pallet, e.name));
	let new_names = news.iter().cloned().map(|e| (e.pallet, e.name));
	let names = old_names.chain(new_names).collect::<std::collections::BTreeSet<_>>();

	for (pallet, extrinsic) in names {
		if !pallet_regex.as_ref().map_or(true, |r| r.is_match(&pallet).unwrap_or_default()) {
			// TODO add "skipped" or "ignored" result type.
			continue
		}
		if !ext_regex.as_ref().map_or(true, |r| r.is_match(&extrinsic).unwrap_or_default()) {
			continue
		}

		let new = news.iter().find(|&n| n.name == extrinsic && n.pallet == pallet);
		let old = olds.iter().find(|&n| n.name == extrinsic && n.pallet == pallet);

		let change = match compare_extrinsics(old, new, method) {
			Err(err) => {
				log::warn!("Parsing failed {}: {:?}", &pallet, err);
				TermDiff::Failed(err)
			},
			Ok(change) => TermDiff::Changed(change),
		};

		diff.push(ExtrinsicDiff { name: extrinsic.clone(), file: pallet.clone(), change });
	}

	Ok(diff)
}

pub fn sort_changes(diff: &mut TotalDiff) {
	diff.sort_by(|a, b| match (&a.change, &b.change) {
		(TermDiff::Failed(_), _) => Ordering::Less,
		(_, TermDiff::Failed(_)) => Ordering::Greater,
		(TermDiff::Changed(a), TermDiff::Changed(b)) => {
			let ord = a.change.cmp(&b.change).reverse();
			if ord == Ordering::Equal {
				if a.percent > b.percent {
					Ordering::Greater
				} else if a.percent == b.percent {
					Ordering::Equal
				} else {
					Ordering::Less
				}
			} else {
				ord
			}
		},
	});
}

pub fn filter_changes(diff: TotalDiff, params: &FilterParams) -> TotalDiff {
	// Note: the pallet and extrinsic are already filtered in compare_files.
	diff.iter()
		.filter(|extrinsic| match extrinsic.change {
			TermDiff::Failed(_) => true,
			TermDiff::Changed(ref change) => match change.change {
				RelativeChange::Changed if change.percent.abs() < params.threshold => false,
				RelativeChange::Unchanged if params.threshold >= 0.001 => false,
				_ => true,
			},
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
