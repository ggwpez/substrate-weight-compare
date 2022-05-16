#[macro_export]
macro_rules! integration_test {
	(
	 	$mod_name:ident,
		$repo:expr,
		$known_good:expr,

		$num_rust_files:expr,
		$num_pallet_files:expr,
		$num_storage_files:expr,
		$num_overhead_files:expr,

		$pallet_patterns:expr,
		$db_patterns:expr,
		$overhead_patterns:expr
	) => {
		mod $mod_name {
			use glob::glob;
			use rstest::*;
			use serial_test::serial;
			use std::path::{Path, PathBuf};

			use $crate::{
				checkout,
				parse::{pallet::parse_file, ParsedFile},
			};

			/// These tests only work on Moonbeam master and are therefore not run by default.
			/// They must possibly be updated on every Moonbeam update.
			mod version_locked {
				use super::*;

				/// The number of rust files in the repo.
				const NUM_RUST_FILES: usize = $num_rust_files;

				/// The number of pallet weight files in the repo.
				const NUM_PALLET_WEIGHT_FILES: usize = $num_pallet_files;

				/// The number of database weight files in the repo.
				const NUM_STORAGE_WEIGHT_FILES: usize = $num_storage_files;

				/// The number of database weight files in the repo.
				const NUM_OVERHEAD_WEIGHT_FILES: usize = $num_overhead_files;

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
				fn num_storage_weight_files() {
					init();
					assert_eq!(storage_files().len(), NUM_STORAGE_WEIGHT_FILES);
				}

				/// Asserts that the correct number of weight files is found.
				#[test]
				#[serial]
				#[cfg_attr(not(all(feature = $repo, feature = "version-locked-tests")), ignore)]
				fn num_overhead_weight_files() {
					init();
					assert_eq!(overhead_files().len(), NUM_OVERHEAD_WEIGHT_FILES);
				}
			}

			/// Parses all weight files successfully.
			#[rstest]
			#[serial]
			#[cfg_attr(not(feature = $repo), ignore)]
			fn parses_pallet_weight_files(pallet_files: Vec<PathBuf>) {
				for file in pallet_files {
					parse_file(Path::new("."), &file)
						.map_err(|e| format!("File {:?}: {:?}", file, e))
						.unwrap();
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
				let weights = rust_files
					.iter()
					.filter(|p| parse_file(Path::new("."), p).is_ok())
					.cloned()
					.collect::<Vec<_>>();

				if pallet_files.len() != weights.len() {
					panic!(
						"Expected {} weights, found {}:\n{}",
						pallet_files.len(),
						weights.len(),
						fmt_diff(&pallet_files, &weights)
					);
				}
			}

			#[rstest]
			#[serial]
			#[cfg_attr(not(feature = $repo), ignore)]
			fn parses_db_weight_files(storage_files: Vec<PathBuf>) {
				for file in storage_files {
					$crate::parse::storage::parse_file(&file).unwrap();
				}
			}

			#[rstest]
			#[serial]
			#[cfg_attr(not(feature = $repo), ignore)]
			fn parses_exactly_db_weight_files(
				rust_files: Vec<PathBuf>,
				storage_files: Vec<PathBuf>,
			) {
				let weights = rust_files
					.iter()
					.filter(|p| $crate::parse::storage::parse_file(p).is_ok())
					.cloned()
					.collect::<Vec<_>>();

				if storage_files.len() != weights.len() {
					panic!(
						"Expected {} weights, found {}:\n{}",
						storage_files.len(),
						weights.len(),
						fmt_diff(&rust_files, &weights)
					);
				}
			}

			/// Test that [`crate::parse::try_parse_file`] detects the correct file type.
			#[rstest]
			#[serial]
			#[cfg_attr(not(feature = $repo), ignore)]
			fn try_parse_detects_correct_files(
				rust_files: Vec<PathBuf>,
				storage_files: Vec<PathBuf>,
				pallet_files: Vec<PathBuf>,
				overhead_files: Vec<PathBuf>,
			) {
				let detected = rust_files
					.iter()
					.filter_map(|f| $crate::parse::try_parse_file(Path::new("."), f))
					.collect::<Vec<_>>();

				let detected_pallets = detected
					.iter()
					.filter(|d| matches!(d, ParsedFile::Pallet(_)))
					.collect::<Vec<_>>();

				if pallet_files.len() != detected_pallets.len() {
					panic!(
						"Expected {} pallet weights files, found {}",
						pallet_files.len(),
						detected_pallets.len(),
					);
				}

				let detected_storage = detected
					.iter()
					.filter(|d| matches!(d, ParsedFile::Storage(_)))
					.collect::<Vec<_>>();

				if storage_files.len() != detected_storage.len() {
					panic!(
						"Expected {} storage weights files, found {}",
						storage_files.len(),
						detected_storage.len(),
					);
				}

				let detected_overhead = detected
					.iter()
					.filter(|d| matches!(d, ParsedFile::Storage(_)))
					.collect::<Vec<_>>();

				if overhead_files.len() != detected_overhead.len() {
					panic!(
						"Expected {} overhead weights files, found {}",
						overhead_files.len(),
						detected_overhead.len(),
					);
				}
			}

			// Setup code

			/// Returns all weight files from a moonbeam repository.
			#[fixture]
			fn pallet_files() -> Vec<PathBuf> {
				let pattern: Vec<&str> = $pallet_patterns;
				pattern
					.iter()
					.map(|pattern| {
						let pattern = format!("{}/{}", root().to_string_lossy(), pattern);
						glob(&pattern)
							.unwrap()
							.map(|f| f.unwrap())
							.filter(|f| !f.ends_with("mod.rs"))
					})
					.flatten()
					.collect()
			}

			/// Returns all weight files from a moonbeam repository.
			#[fixture]
			fn storage_files() -> Vec<PathBuf> {
				let pattern: Vec<&str> = $db_patterns;
				pattern
					.iter()
					.map(|pattern| {
						let pattern = format!("{}/{}", root().to_string_lossy(), pattern);
						glob(&pattern).unwrap().map(|f| f.unwrap())
					})
					.flatten()
					.collect()
			}

			// FIXME remove moonbeam

			/// Returns all weight files from a moonbeam repository.
			#[fixture]
			fn overhead_files() -> Vec<PathBuf> {
				let pattern: Vec<&str> = $overhead_patterns;
				pattern
					.iter()
					.map(|pattern| {
						let pattern = format!("{}/{}", root().to_string_lossy(), pattern);
						glob(&pattern).unwrap().map(|f| f.unwrap())
					})
					.flatten()
					.collect()
			}

			/// Returns the number of rust files in the Moonbeam repository.
			#[fixture]
			fn rust_files() -> Vec<PathBuf> {
				let root = format!("{}/**/*.rs", root().to_string_lossy());
				glob(&root).unwrap().map(|f| f.unwrap()).collect()
			}

			/// Returns the root directory to the Moonbeam git repository.
			fn root() -> PathBuf {
				PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("repos").join($repo)
			}

			/// Format all files that are not pallet files and all pallet files that are not files.
			fn fmt_diff(files: &[PathBuf], weights: &[PathBuf]) -> String {
				let mut output = String::new();
				for weight in weights.iter() {
					if !files.contains(weight) {
						output.push_str("File could unexpectedly be parsed: ");
						output.push_str(&weight.display().to_string());
						output.push_str("\n");
					}
				}
				output
			}
		}
	};
}
