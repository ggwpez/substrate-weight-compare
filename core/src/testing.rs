use semver::Version;
use std::{
	ops::{Deref, DerefMut},
	path::PathBuf,
	process::Child,
};

pub fn root_dir() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

pub fn assert_version(raw: &str, name: &str) {
	let split = raw.split(' ').collect::<Vec<_>>();
	assert_eq!(split.len(), 2);
	let version = split[1];

	assert_eq!(split[0], name);
	assert!(Version::parse(version).is_ok(), "Version should be a valid Semver");
	assert_eq!(version, &*crate::VERSION, "Wrong version string");
}

/// Asserts that the command output is successful.
// TODO: Could be done as extension trait.
pub fn succeeds(output: &std::process::Output) {
	if !output.status.success() {
		panic!("{}", String::from_utf8_lossy(&output.stderr));
	}
}

pub fn assert_contains(output: &str, pattern: &str) {
	if !output.contains(pattern) {
		panic!("The output:\n{:?}\nDid not contain the pattern:\n{:?}", output, pattern);
	}
}

pub fn assert_not_contains(output: &str, pattern: &str) {
	if output.contains(pattern) {
		panic!("The output:\n{:?}\nDid contain the pattern:\n{:?}", output, pattern);
	}
}

pub struct KillChildOnDrop(pub Child);

impl Drop for KillChildOnDrop {
	fn drop(&mut self) {
		let _ = self.0.kill();
	}
}

impl Deref for KillChildOnDrop {
	type Target = Child;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for KillChildOnDrop {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
