use clap::Parser;

use std::path::{Path, PathBuf};

use swc_core::{
	compare_commits, compare_files, fmt_changes,
	parse::{pallet::parse_files, try_parse_file},
	CompareParams, TotalDiff, VERSION,
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
			let olds = parse_files(&Path::new("."), &old)?;
			let news = parse_files(&Path::new("."), &new)?;

			let diff = compare_files(olds, news, params.threshold, params.method);
			print_changes(diff, cmd.verbose);
		},
		SubCommand::Compare(CompareCmd::Commits(CompareCommitsCmd { old, new, params, repo, path_pattern })) => {
			let per_extrinsic = compare_commits(
				&repo,
				&old,
				&new,
				params.threshold,
				params.method,
				&path_pattern,
				params.ignore_errors,
				usize::MAX,
			)?;
			print_changes(per_extrinsic, cmd.verbose);
		},
		SubCommand::Parse(ParseCmd::Files(ParseFilesCmd { files })) => {
			println!("Trying to parse {} files...", files.len());
			let parsed = files.iter().filter_map(|f| try_parse_file(&Path::new("."), f));
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

	print(format!("\n{}", fmt_changes(&per_extrinsic)), verbose);
}

fn print(msg: String, verbose: bool) {
	if verbose {
		log::info!("{}", msg);
	} else {
		println!("{}", msg);
	}
}
