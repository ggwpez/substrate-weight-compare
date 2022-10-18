use clap::{Args, Parser};
use comfy_table::Table;
use std::{fmt::Write as _, path::PathBuf};

use swc_core::{
	compare_commits, compare_files, filter_changes,
	parse::pallet::{parse_files, try_parse_files},
	sort_changes, CompareParams, FilterParams, Percent, RelativeChange, TotalDiff, Unit, VERSION,
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

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub filter: FilterParams,

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub format: FormatParams,

	/// The old weight files.
	#[clap(long, required(true), multiple_values(true))]
	pub old: Vec<PathBuf>,

	/// The new weight files.
	#[clap(long, required(true), multiple_values(true))]
	pub new: Vec<PathBuf>,
}

#[derive(Debug, Parser)]
struct CompareCommitsCmd {
	#[allow(missing_docs)]
	#[clap(flatten)]
	pub params: CompareParams,

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub filter: FilterParams,

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub format: FormatParams,

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

#[derive(Debug, Parser)]
struct ParseFilesCmd {
	/// The files to parse.
	#[clap(long, index = 1, required(true), multiple_values(true))]
	pub files: Vec<PathBuf>,
}

/// Parameters for modifying the output representation.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct FormatParams {
	/// Set the format of the output.
	#[clap(long, value_name  ="FORMAT", default_value = "human", ignore_case = true,
possible_values = OutputFormat::variants())]
	pub format: OutputFormat,

	/// Include weight terms in the console output.
	///
	/// Note: The output will have _very_ long rows.
	#[clap(long)]
	print_terms: bool,

	/// Disable color output.
	#[clap(long)]
	no_color: bool,
}

#[derive(Debug, serde::Deserialize, clap::ArgEnum, Clone, Eq, Ord, PartialEq, PartialOrd, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
	/// Full human readable output.
	Human,
	/// Condensed human readable output.
	BriefHuman,
	/// CSV (comma separated values) list.
	CSV,
	/// Json output.
	JSON,
}

impl OutputFormat {
	/// All possible variants of [`Self`].
	pub fn variants() -> Vec<&'static str> {
		vec!["human", "brief-human", "csv", "json"]
	}
}

impl std::str::FromStr for OutputFormat {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"human" => Ok(OutputFormat::Human),
			"brief-human" => Ok(OutputFormat::BriefHuman),
			"csv" => Ok(OutputFormat::CSV),
			"json" => Ok(OutputFormat::JSON),
			_ => Err(format!("Unknown output format: {}", s)),
		}
	}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let cmd = MainCmd::parse();

	// TODO is is good to not set this up at all?!
	if cmd.verbose {
		env_logger::init_from_env(
			env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
		);
	}

	match cmd.subcommand {
		SubCommand::Compare(CompareCmd::Files(CompareFilesCmd {
			params,
			filter,
			format,
			old,
			new,
		})) => {
			let olds =
				if params.ignore_errors { try_parse_files(&old) } else { parse_files(&old)? };
			let news =
				if params.ignore_errors { try_parse_files(&new) } else { parse_files(&new)? };

			let mut diff = compare_files(olds, news, params.method, &filter, params.ignore_errors)?;
			diff = filter_changes(diff, &filter)?;
			sort_changes(&mut diff);
			print_changes(diff, cmd.verbose, format, params.unit)?;
		},
		SubCommand::Compare(CompareCmd::Commits(CompareCommitsCmd {
			params,
			filter,
			format,
			old,
			new,
			repo,
			path_pattern,
		})) => {
			let mut diff =
				compare_commits(&repo, &old, &new, &params, &filter, &path_pattern, usize::MAX)?;
			diff = filter_changes(diff, &filter)?;
			sort_changes(&mut diff);
			print_changes(diff, cmd.verbose, format, params.unit)?;
		},
		SubCommand::Parse(ParseCmd::Files(ParseFilesCmd { files })) => {
			println!("Trying to parse {} files...", files.len());
			let parsed = parse_files(&files)?;
			println!("Parsed {} files successfully", parsed.len());
		},
	}

	Ok(())
}

fn print_changes(
	per_extrinsic: TotalDiff,
	verbose: bool,
	format: FormatParams,
	unit: Unit,
) -> Result<(), Box<dyn std::error::Error>> {
	let output = match format.format {
		OutputFormat::Human => print_changes_human(per_extrinsic, verbose, format, unit),
		OutputFormat::CSV => print_changes_csv(per_extrinsic, verbose, format, unit),
		_ => Err("Unsupported output format".into()),
	};

	print(output?, verbose);
	Ok(())
}

// TODO make meta output format
fn print_changes_csv(
	per_extrinsic: TotalDiff,
	verbose: bool,
	format: FormatParams,
	unit: Unit,
) -> Result<String, Box<dyn std::error::Error>> {
	if per_extrinsic.is_empty() {
		print("No changes found.".into(), verbose);
		return Ok(String::new())
	}

	let mut output = String::new();
	// Put a csv header
	output.push_str("File,Extrinsic,Old,New,Change Percent");
	if format.print_terms {
		output.push_str(",Old Weight Term,New Weight Term,Used variables");
	}
	output.push('\n');

	for change in per_extrinsic.iter() {
		let mut row = format!(
			"{},{},{},{},{},",
			change.file.clone(),
			change.name.clone(),
			change.change.old_v.map(|v| unit.fmt_value(v)).unwrap_or_default(),
			change.change.new_v.map(|v| unit.fmt_value(v)).unwrap_or_default(),
			color_percent(change.change.percent, &change.change.change, format.no_color),
		);

		if format.print_terms {
			write!(
				row,
				"{},",
				change
					.change
					.old
					.as_ref()
					.map(|t| format!("{}", t))
					.unwrap_or_else(|| "-".into())
			)?;
			write!(
				row,
				"{},",
				change
					.change
					.new
					.as_ref()
					.map(|t| format!("{}", t))
					.unwrap_or_else(|| "-".into())
			)?;
			row.push_str(&format!("{:?}", &change.change.scope).replace(',', " "));
		}
		row.push('\n');
		output.push_str(&row);
	}

	Ok(output)
}

fn print_changes_human(
	per_extrinsic: TotalDiff,
	verbose: bool,
	format: FormatParams,
	unit: Unit,
) -> Result<String, Box<dyn std::error::Error>> {
	if per_extrinsic.is_empty() {
		print("No changes found.".into(), verbose);
		return Ok(String::new())
	}

	let mut table = Table::new();
	table.set_constraints(vec![comfy_table::ColumnConstraint::ContentWidth]);
	let mut header = vec!["File", "Extrinsic", "Old", "New", "Change [%]"];
	if format.print_terms {
		header.extend(vec!["Old Weight Term", "New Weight Term", "Used variables"]);
	}
	table.set_header(header);

	for change in per_extrinsic.iter() {
		let mut row = vec![
			change.file.clone(),
			change.name.clone(),
			change.change.old_v.map(|v| unit.fmt_value(v)).unwrap_or_default(),
			change.change.new_v.map(|v| unit.fmt_value(v)).unwrap_or_default(),
			color_percent(change.change.percent, &change.change.change, format.no_color),
		];

		if format.print_terms {
			row.extend(vec![
				change
					.change
					.old
					.as_ref()
					.map(|t| format!("{}", t))
					.unwrap_or_else(|| "-".into()),
				change
					.change
					.new
					.as_ref()
					.map(|t| format!("{}", t))
					.unwrap_or_else(|| "-".into()),
				format!("{:?}", &change.change.scope),
			]);
		}
		table.add_row(row);
	}
	Ok(table.to_string())
}

fn print(msg: String, verbose: bool) {
	if verbose {
		log::info!("{}", msg);
	} else {
		println!("{}", msg);
	}
}

enum AnsiColor {
	White,
	Red,
	Green,
}

pub fn color_percent(p: Percent, change: &RelativeChange, no_color: bool) -> String {
	match change {
		RelativeChange::Unchanged => "Unchanged".to_string(),
		RelativeChange::Added => maybe_color(AnsiColor::Red, "Added", no_color),
		RelativeChange::Removed => maybe_color(AnsiColor::Green, "Removed", no_color),
		RelativeChange::Changed => {
			let s = format!("{:+5.2}", p);
			match p {
				x if x < 0.0 => maybe_color(AnsiColor::Green, s, no_color),
				x if x > 0.0 => maybe_color(AnsiColor::Red, s, no_color),
				_ => maybe_color(AnsiColor::White, s, no_color),
			}
		},
	}
}

impl AnsiColor {
	fn paint(&self, s: &str) -> String {
		match self {
			AnsiColor::White => format!("\x1b[37m{}\x1b[0m", s),
			AnsiColor::Red => format!("\x1b[31m{}\x1b[0m", s),
			AnsiColor::Green => format!("\x1b[32m{}\x1b[0m", s),
		}
	}
}

fn maybe_color<S: Into<String>>(clr: AnsiColor, msg: S, no_color: bool) -> String {
	let msg = msg.into();
	if no_color {
		msg
	} else {
		clr.paint(&msg)
	}
}
