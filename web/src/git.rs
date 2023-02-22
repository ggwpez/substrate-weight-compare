//! Helper functions for dealing with the git CLI.

use fancy_regex::Regex;
use std::{path::Path, process::Command};

/// Returns the GitHub organization name for a given repository.
///
/// Yes this is inflexible and depends on GitHub - whatever it works.
pub fn get_origin_org(repo: &Path) -> Result<String, String> {
	let output = Command::new("git")
		.args(["remote", "get-url", "origin"])
		.current_dir(repo)
		.output()
		.map_err(|e| format!("Failed to get origin url: {}", e))?;
	if !output.status.success() {
		return Err(format!("Failed to get origin url: {}", String::from_utf8_lossy(&output.stderr)))
	}
	let regex = Regex::new(r"^https://github.com/([^/]+)/").unwrap();
	regex
		.captures(&String::from_utf8_lossy(&output.stdout))
		.map_err(|e| format!("Failed to parse origin url: {}", e))?
		.ok_or_else(|| "Failed to parse origin url".to_string())
		.map(|c| c.get(1).unwrap().as_str().to_string())
}

#[cfg(test)]
mod tests {
	use super::*;
	use rstest::*;

	#[rstest]
	#[case("https://github.com/myorg/myrepo", Some("myorg"))]
	#[case("https://github.com/myorg/myrepo.git", Some("myorg"))]
	#[case("https://gitlab.com/myorg/myrepo", None)]
	#[case("ssh://gitlab.com:myorg/myrepo", None)]
	fn get_origin_org_works(#[case] url: &str, #[case] org: Option<&str>) {
		// Create a temporary directory.
		let tmp = tempfile::tempdir().unwrap();
		// Create a new git repo in the temporary directory.
		Command::new("git").args(["init"]).current_dir(tmp.path()).output().unwrap();
		// Set the origin to a dummy URL.
		Command::new("git")
			.args(["remote", "add", "origin", url])
			.current_dir(tmp.path())
			.output()
			.unwrap();
		// Check that the origin org is as expected.
		assert_eq!(org.map(Into::into), get_origin_org(tmp.path()).ok());
	}
}
