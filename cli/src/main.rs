use clap::Parser;
use comfy_table::Table;
use std::path::PathBuf;

use swc_core::{
	compare_commits, compare_files, filter_changes, fmt_weight,
	parse::pallet::{parse_files, try_parse_files},
	sort_changes, CompareParams, FilterParams, Percent, RelativeChange, TotalDiff, VERSION,
};

#[derive(Debug, Parser)]
#[clap(author, version(&VERSION[..]))]
struct MainCmd {
	#[clap(subcommand)]
	subcommand: SubCommand,

	#[clap(long)]
	verbose: bool,

	/// Disable color output.
	#[clap(long)]
	no_color: bool,
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

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub filter: FilterParams,

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

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub filter: FilterParams,

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
		SubCommand::Compare(CompareCmd::Files(CompareFilesCmd { old, new, params, filter })) => {
			let olds =
				if params.ignore_errors { try_parse_files(&old) } else { parse_files(&old)? };
			let news =
				if params.ignore_errors { try_parse_files(&new) } else { parse_files(&new)? };

			let mut diff = compare_files(olds, news, params.method);
			diff = filter_changes(diff, &filter);
			sort_changes(&mut diff);
			print_changes(diff, cmd.verbose, cmd.no_color);
		},
		SubCommand::Compare(CompareCmd::Commits(CompareCommitsCmd {
			old,
			new,
			params,
			filter,
			repo,
			path_pattern,
		})) => {
			let mut diff = compare_commits(&repo, &old, &new, &params, &path_pattern, usize::MAX)?;
			diff = filter_changes(diff, &filter);
			print_changes(diff, cmd.verbose, cmd.no_color);
		},
		SubCommand::Parse(ParseCmd::Files(ParseFilesCmd { files })) => {
			println!("Trying to parse {} files...", files.len());
			let parsed = try_parse_files(&files);
			println!("Parsed {} files successfully", parsed.len());
		},
	}

	Ok(())
}

fn print_changes(per_extrinsic: TotalDiff, verbose: bool, no_color: bool) {
	if per_extrinsic.is_empty() {
		print("No changes found.".into(), verbose);
		return
	}

	let mut table = Table::new();
	table.set_header(vec!["File", "Extrinsic", "Old", "New", "Change [%]"]);

	for change in per_extrinsic.iter() {
		table.add_row(vec![
			change.file.clone(),
			change.name.clone(),
			change.change.old_v.map(fmt_weight).unwrap_or_default(),
			change.change.new_v.map(fmt_weight).unwrap_or_default(),
			color_percent(change.change.percent, &change.change.change, no_color),
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

pub fn color_percent(p: Percent, change: &RelativeChange, no_color: bool) -> String {
	use ansi_term::Colour;

	match change {
		RelativeChange::Unchanged => "0.00% (No change)".to_string(),
		RelativeChange::Added => maybe_color(Colour::Red, "+100.00% (Added)", no_color),
		RelativeChange::Removed => maybe_color(Colour::Green, "-100.00% (Removed)", no_color),
		RelativeChange::Changed => {
			let s = format!("{:+5.2}", p);
			match p {
				x if x < 0.0 => maybe_color(Colour::Green, s, no_color),
				x if x > 0.0 => maybe_color(Colour::Red, s, no_color),
				_ => maybe_color(Colour::White, s, no_color),
			}
		},
	}
}

fn maybe_color<S: Into<String>>(clr: ansi_term::Colour, msg: S, no_color: bool) -> String {
	if no_color {
		msg.into()
	} else {
		clr.paint(msg.into()).to_string()
	}
}
