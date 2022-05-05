use clap::Parser;
use std::path::PathBuf;

use swc::{parse::*, *};

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
}

#[derive(Debug, clap::Subcommand)]
enum CompareCmd {
    Files(CompareFilesCmd),
    Commits(CompareCommitsCmd),
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
            let olds = parse_files(&old, &params.blacklist_file)?;
            let news = parse_files(&new, &params.blacklist_file)?;
            let diff = compare_files(olds, news);
            let per_extrinsic = extract_changes(diff, params.threshold);
            print_changes(per_extrinsic, cmd.verbose);
        }
        SubCommand::Compare(CompareCmd::Commits(CompareCommitsCmd { old, new, params })) => {
            let per_extrinsic =
                compare_commits(&old, &new, params.threshold, params.blacklist_file)?;
            print_changes(per_extrinsic, cmd.verbose);
        }
    }

    Ok(())
}

fn print_changes(per_extrinsic: Vec<ExtrinsicDiff>, verbose: bool) {
    if per_extrinsic.is_empty() {
        print("No changes found.".into(), verbose);
        return;
    }

    for line in fmt_changes(&per_extrinsic) {
        print(line, verbose);
    }
}

fn print(msg: String, verbose: bool) {
    if verbose {
        log::info!("{}", msg);
    } else {
        println!("{}", msg);
    }
}
