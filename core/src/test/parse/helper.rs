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

		$pallet_patterns:expr; exclude=$exclude_pallet:expr,
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
				parse::ParsedFile,
			};

			/// These tests only work on master and are therefore not run by default.
			/// They must possibly be updated on every master update.
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

				/// Ensure that master is checked out.
				///
				/// Other tests could have messed it up.
				fn init() {
					if let Err(err) = checkout(&root(), $known_good, false) {
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

				/// Asserts that the correct number of pallet weight files is found.
				#[test]
				#[serial]
				#[cfg_attr(not(all(feature = $repo, feature = "version-locked-tests")), ignore)]
				fn num_pallet_weight_files() {
					init();
					assert_eq!(pallet_files().len(), NUM_PALLET_WEIGHT_FILES);
				}

				/// Asserts that the correct number of storage weight files is found.
				#[test]
				#[serial]
				#[cfg_attr(not(all(feature = $repo, feature = "version-locked-tests")), ignore)]
				fn num_storage_weight_files() {
					init();
					assert_eq!(storage_files().len(), NUM_STORAGE_WEIGHT_FILES);
				}

				/// Asserts that the correct number of overhead weight files is found.
				#[test]
				#[serial]
				#[cfg_attr(not(all(feature = $repo, feature = "version-locked-tests")), ignore)]
				fn num_overhead_weight_files() {
					init();
					assert_eq!(overhead_files().len(), NUM_OVERHEAD_WEIGHT_FILES);
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
				for f in &rust_files {
					match $crate::parse::try_parse_file(Path::new("."), f){
						None => if pallet_files.contains(f) {
							let err = $crate::parse::pallet::parse_file(f).unwrap_err();
							assert!(false, "File {:?} could not be parsed as pallet: {:?}", f, err)
						} else if overhead_files.contains(f) {
							let err = $crate::parse::overhead::parse_file(f).unwrap_err();
							assert!(false, "File {:?} could not be parsed as overhead: {:?}", f, err)
						} else if storage_files.contains(f) {
							let err = $crate::parse::storage::parse_file(f).unwrap_err();
							assert!(false, "File {:?} could not be parsed as storage: {:?}", f, err)
						},
						Some(ParsedFile::Pallet(_)) => if !pallet_files.contains(f) {
							assert!(false, "File {:?} was parsed as pallet, but it was not expected to be", f)
						},
						Some(ParsedFile::Overhead(_)) => if !overhead_files.contains(f) {
							assert!(false, "File {:?} was parsed as overhead, but it was not expected to be", f)
						},
						Some(ParsedFile::Storage(_)) => if !storage_files.contains(f) {
							assert!(false, "File {:?} was parsed as storage, but it was not expected to be", f)
						},
					}
				}
			}

			// Setup code

			/// Returns all weight files from the repository.
			#[fixture]
			fn pallet_files() -> Vec<PathBuf> {
				let pattern: Vec<&str> = $pallet_patterns;
				let exclude: Vec<&str> = $exclude_pallet;
				pattern
					.iter()
					.map(|pattern| {
						let pattern = format!("{}/{}", root().to_string_lossy(), pattern);
						glob(&pattern)
							.unwrap()
							.map(|f| f.unwrap())
							.filter(|f| !f.ends_with("mod.rs"))
							.filter(|f| !exclude.iter().any(|n| f.to_string_lossy().ends_with(n))
						)
					})
					.flatten()
					.collect()
			}

			/// Returns all weight files from the repository.
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

			/// Returns all weight files from the repository.
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

			/// Returns the number of rust files in the repository.
			#[fixture]
			fn rust_files() -> Vec<PathBuf> {
				let root = format!("{}/**/*.rs", root().to_string_lossy());
				glob(&root).unwrap().map(|f| f.unwrap()).collect()
			}

			/// Returns the root directory to the git repository.
			fn root() -> PathBuf {
				PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("repos").join($repo)
			}
		}
	};
}
