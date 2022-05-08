#[macro_export]
macro_rules! integration_test {
	(
	 	$mod_name:ident,
		$repo:expr,
		$known_good:expr,

		$num_rust_files:expr,
		$num_pallet_files:expr,
		$num_db_files:expr,

		$pallet_pattern:expr,
		$db_pattern:expr
	) => {
		mod $mod_name {
			use glob::glob;
			use rstest::*;
			use serial_test::serial;
			use std::path::PathBuf;

			use swc::{checkout, parse::pallet::parse_file};

			/// These tests only work on Moonbeam master and are therefore not run by default.
			/// They must possibly be updated on every Moonbeam update.
			mod version_locked {
				use super::*;

				/// The number of rust files in the repo.
				const NUM_RUST_FILES: usize = $num_rust_files;

				/// The number of pallet weight files in the repo.
				const NUM_PALLET_WEIGHT_FILES: usize = $num_pallet_files;

				/// The number of database weight files in the repo.
				const NUM_DB_WEIGHT_FILES: usize = $num_db_files;

				/// Ensure that Moonbeam master is checked out.
				///
				/// Other tests could have messed it up.
				fn init() {
					if let Err(err) = checkout(&root(), $known_good) {
						panic!("Could not check out `repos/{}` to: {}", $repo, err);
					}
				}

				/// Asserts that the correct number of rust files is found.
				#[test]
				#[serial]
				#[cfg_attr(not(all(feature = $repo, feature = "version-locked-tests")), ignore)]
				fn num_rust_files() {
					init();
					assert_eq!(rust_files().len(), NUM_RUST_FILES);
				}

				/// Asserts that the correct number of weight files is found.
				#[test]
				#[serial]
				#[cfg_attr(not(all(feature = $repo, feature = "version-locked-tests")), ignore)]
				fn num_pallet_weight_files() {
					init();
					assert_eq!(pallet_files().len(), NUM_PALLET_WEIGHT_FILES);
				}

				/// Asserts that the correct number of weight files is found.
				#[test]
				#[serial]
				#[cfg_attr(not(all(feature = $repo, feature = "version-locked-tests")), ignore)]
				fn num_db_weight_files() {
					init();
					assert_eq!(db_files().len(), NUM_DB_WEIGHT_FILES);
				}
			}

			/// Parses all weight files successfully.
			#[rstest]
			#[serial]
			#[cfg_attr(not(feature = $repo), ignore)]
			fn parses_pallet_weight_files(pallet_files: Vec<PathBuf>) {
				for file in pallet_files {
					parse_file(&file).map_err(|e| format!("File {:?}: {:?}", file, e)).unwrap();
				}
			}

			/// Tries to parse all rust files and asserts that the number of successful parses is
			/// equal to the number of weight files.
			// TODO: Check for equality instead of just length.
			#[rstest]
			#[serial]
			#[cfg_attr(not(feature = $repo), ignore)]
			fn parses_exactly_pallet_weight_files(
				rust_files: Vec<PathBuf>,
				pallet_files: Vec<PathBuf>,
			) {
				let weights = rust_files.iter().map(|p| parse_file(p)).filter_map(|r| r.ok());

				assert_eq!(weights.count(), pallet_files.len());
			}

			#[rstest]
			#[serial]
			#[cfg_attr(not(feature = $repo), ignore)]
			fn parses_db_weight_files(db_files: Vec<PathBuf>) {
				for file in db_files {
					swc::parse::storage::parse_file(&file).unwrap();
				}
			}

			#[rstest]
			#[serial]
			#[cfg_attr(not(feature = $repo), ignore)]
			fn parses_exactly_db_weight_files(rust_files: Vec<PathBuf>, db_files: Vec<PathBuf>) {
				let weights = rust_files
					.iter()
					.map(|p| swc::parse::storage::parse_file(p))
					.filter_map(|r| r.ok());

				assert_eq!(weights.count(), db_files.len());
			}

			// Setup code

			/// Returns all weight files from a moonbeam repository.
			#[fixture]
			fn pallet_files() -> Vec<PathBuf> {
				let root = format!("{}/{}", root().to_string_lossy(), $pallet_pattern);
				glob(&root)
					.unwrap()
					.map(|f| f.unwrap())
					.filter(|f| !f.ends_with("mod.rs"))
					.collect()
			}

			/// Returns all weight files from a moonbeam repository.
			#[fixture]
			fn db_files() -> Vec<PathBuf> {
				let root = format!("{}/{}", root().to_string_lossy(), $db_pattern);
				glob(&root).unwrap().map(|f| f.unwrap()).collect()
			}

			/// Returns the number of rust files in the Moonbeam repository.
			#[fixture]
			fn rust_files() -> Vec<PathBuf> {
				let root = format!("{}/**/*.rs", root().to_string_lossy());
				glob(&root).unwrap().map(|f| f.unwrap()).collect()
			}

			/// Returns the root directory to the Moonbeam git repository.
			fn root() -> PathBuf {
				PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("repos").join($repo)
			}
		}
	};
}
