use clap::Parser;
use substrate_weight_compare::*;

#[derive(Debug, Parser)]
struct MainCmd {
    #[allow(missing_docs)]
    #[clap(flatten)]
    pub compare_params: CompareParams,
}

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let params = MainCmd::parse().compare_params;
    let diff = compare_files(&params);
    let per_extrinsic = extract_changes(&params, diff);
    for line in fmt_changes(&per_extrinsic) {
        log::info!("{}", line);
    }
}
