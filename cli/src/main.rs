use clap::Parser;
use prettytable::{cell, row, table};
use std::path::{Path, PathBuf};

use swc_core::{
	compare_commits, compare_files, fmt_weight,
	parse::{pallet::parse_files, try_parse_file},
	sort_changes, CompareParams, Percent, RelativeChange, TotalDiff, VERSION,
};

#[derive(Debug, Parser)]
#[clap(author, version(&VERSION[..]))]
struct MainCmd {
	#[clap(subcommand)]
	subcommand: SubCommand,

	#[clap(long)]
	verbose: bool,
}

#[derive(Debug, clap::Subcommand)]
enum SubCommand {
	#[clap(subcommand)]
	Compare(CompareCmd),
	#[clap(subcommand)]
	Parse(ParseCmd),
}

#[derive(Debug, clap::Subcommand)]
enum CompareCmd {
	Files(CompareFilesCmd),
	Commits(CompareCommitsCmd),
}

/// Tries to parse all files in the given file list or folder.
#[derive(Debug, clap::Subcommand)]
enum ParseCmd {
	Files(ParseFilesCmd),
}

#[derive(Debug, Parser)]
struct CompareFilesCmd {
	#[allow(missing_docs)]
	#[clap(flatten)]
	pub params: CompareParams,

	/// The old weight files.
	#[clap(long, required(true), multiple_values(true))]
	pub old: Vec<PathBuf>,

	/// The new weight files.
	#[clap(long, required(true), multiple_values(true))]
	pub new: Vec<PathBuf>,
}

#[derive(Debug, Parser)]
struct ParseFilesCmd {
	/// The files to parse.
	#[clap(long, index = 1, required(true), multiple_values(true))]
	pub files: Vec<PathBuf>,
}

#[derive(Debug, Parser)]
struct CompareCommitsCmd {
	#[allow(missing_docs)]
	#[clap(flatten)]
	pub params: CompareParams,

	/// Old commit/branch/tag.
	#[clap(name = "OLD-COMMIT", index = 1)]
	pub old: String,

	/// New commit/branch/tag.
	#[clap(name = "NEW-COMMIT", index = 2, default_value = "master")]
	pub new: String,

	#[clap(long, default_value = "repos/polkadot")]
	pub repo: PathBuf,

	#[clap(long)]
	pub path_pattern: String,
}

fn main() -> Result<(), String> {
	let cmd = MainCmd::parse();

	// TODO is is good to not set this up at all?!
	if cmd.verbose {
		env_logger::init_from_env(
			env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
		);
	}

	match cmd.subcommand {
		SubCommand::Compare(CompareCmd::Files(CompareFilesCmd { old, new, params })) => {
			let olds = parse_files(&old)?;
			let news = parse_files(&new)?;

			let mut diff = compare_files(olds, news, params.threshold, params.method);
			sort_changes(&mut diff);
			print_changes(diff, cmd.verbose);
		},
		SubCommand::Compare(CompareCmd::Commits(CompareCommitsCmd {
			old,
			new,
			params,
			repo,
			path_pattern,
		})) => {
			let per_extrinsic =
				compare_commits(&repo, &old, &new, &params, &path_pattern, usize::MAX)?;
			print_changes(per_extrinsic, cmd.verbose);
		},
		SubCommand::Parse(ParseCmd::Files(ParseFilesCmd { files })) => {
			println!("Trying to parse {} files...", files.len());
			let parsed = files.iter().filter_map(|f| try_parse_file(Path::new("."), f));
			println!("Parsed {} files successfully", parsed.count());
		},
	}

	Ok(())
}

fn print_changes(per_extrinsic: TotalDiff, verbose: bool) {
	if per_extrinsic.is_empty() {
		print("No changes found.".into(), verbose);
		return
	}

	let mut table = table!(["File", "Extrinsic", "Old", "New", "Change [%]"]);

	for change in per_extrinsic.iter() {
		table.add_row(row![
			change.file,
			change.name,
			change.change.old_v.map(fmt_weight).unwrap_or_default(),
			change.change.new_v.map(fmt_weight).unwrap_or_default(),
			color_percent(change.change.percent, &change.change.change),
		]);
	}
	print(table.to_string(), verbose)
}

fn print(msg: String, verbose: bool) {
	if verbose {
		log::info!("{}", msg);
	} else {
		println!("{}", msg);
	}
}

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
