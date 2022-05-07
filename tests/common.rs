use semver::Version;
use std::{
	ops::{Deref, DerefMut},
	process::Child,
};

pub fn valid_version(raw: &str) {
	let split = raw.split(' ').collect::<Vec<_>>();
	assert_eq!(split.len(), 2);
	let version = split[1];

	assert_eq!(split[0], "swc");
	assert!(Version::parse(version).is_ok(), "Version should be a valid Semver");
	assert_eq!(version, *swc::VERSION, "Wrong version string");
}

/// Asserts that the command output is successful.
// TODO: Could be done as extension trait.
pub fn succeeds(output: &std::process::Output) {
	if !output.status.success() {
		panic!("{}", String::from_utf8_lossy(&output.stderr));
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
