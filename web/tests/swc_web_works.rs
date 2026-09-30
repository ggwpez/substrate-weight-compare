#![cfg(test)]

use assert_cmd::cargo::CommandCargoExt;
use serial_test::serial;
use std::process::Command;

use subweight_core::testing::{assert_version, root_dir, succeeds, KillChildOnDrop};

#[test]
fn subweight_web_version_works() {
	let output = Command::cargo_bin("subweight-web").unwrap().arg("--version").output().unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert_version(&out, "subweight-web");
}

#[test]
fn subweight_web_help_works() {
	let output = Command::cargo_bin("subweight-web").unwrap().arg("--help").output().unwrap();
	succeeds(&output);

	let out = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	assert!(out.contains("Print help"));
}

#[cfg(unix)]
fn command_with_open_file_limit(soft: libc::rlim_t) -> Command {
	use std::os::unix::process::CommandExt;
	let mut command = Command::cargo_bin("subweight-web").unwrap();
	// SAFETY: the child hook only calls async-signal-safe getrlimit/setrlimit;
	// it changes the child process's soft limit, never the test runner's.
	unsafe {
		command.pre_exec(move || {
			let mut limit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
			if libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) != 0 {
				return Err(std::io::Error::last_os_error())
			}
			limit.rlim_cur = soft;
			if libc::setrlimit(libc::RLIMIT_NOFILE, &limit) != 0 {
				return Err(std::io::Error::last_os_error())
			}
			Ok(())
		});
	}
	command
}

#[test]
#[cfg(unix)]
fn startup_checks_descriptor_limit_before_repositories() {
	let temp = tempfile::tempdir().unwrap();
	let missing_root = temp.path().join("missing");
	for soft in [1024, 65_535, 65_536] {
		let output = command_with_open_file_limit(soft)
			.arg("--root")
			.arg(&missing_root)
			.output()
			.unwrap();
		assert!(!output.status.success());
		let error = String::from_utf8_lossy(&output.stderr);
		if soft < 65_536 {
			assert!(error.contains(&format!("Open-file soft limit is {soft}")), "{error}");
			assert!(error.contains("requires at least 65536"), "{error}");
			assert!(error.contains("ulimit -Sn 65536"), "{error}");
			assert!(!error.contains("Root path"), "{error}");
		} else {
			assert!(error.contains("Root path"), "{error}");
			assert!(!error.contains("Open-file soft limit"), "{error}");
		}
	}
}

#[test]
#[cfg(unix)]
fn help_and_version_work_with_low_descriptor_limit() {
	for flag in ["--help", "--version"] {
		let output = command_with_open_file_limit(1024).arg(flag).output().unwrap();
		succeeds(&output);
	}
}

fn server_command() -> Command {
	#[cfg(unix)]
	let command = command_with_open_file_limit(65_536);
	#[cfg(not(unix))]
	let command = Command::cargo_bin("subweight-web").unwrap();
	command
}

#[test]
#[serial]
#[cfg_attr(not(feature = "polkadot"), ignore)]
fn subweight_web_url_works() {
	let _cmd = KillChildOnDrop(
		server_command()
			.args([
				"--root",
				root_dir().join("repos").to_str().unwrap(),
				"--repos",
				"polkadot",
				"--static",
				"../web/static",
			])
			.env("RUST_LOG", "error")
			.spawn()
			.unwrap(),
	);

	for _ in 0..20 {
		std::thread::sleep(std::time::Duration::from_millis(100));

		let Ok(resp) =
			reqwest::blocking::get("http://127.0.0.1:8080/compare").map(|r| r.text().unwrap())
		else {
			continue
		};

		// Search for an example:
		if resp.contains("Polkadot with tags") {
			return
		}
	}
	panic!("Failed to make request in time");
}

#[test]
#[serial]
#[cfg_attr(not(feature = "polkadot"), ignore)]
fn subweight_web_compare_works() {
	let _cmd = KillChildOnDrop(
		server_command()
			.args([
				"--root",
				root_dir().join("repos").to_str().unwrap(),
				"--repos",
				"polkadot",
				"--static",
				"../web/static",
				"--port",
				"4000",
			])
			.env("RUST_LOG", "error")
			.spawn()
			.unwrap(),
	);

	for _ in 0..20 {
		std::thread::sleep(std::time::Duration::from_millis(100));

		let url = "http://127.0.0.1:4000/compare?old=v0.9.19&new=v0.9.20&repo=polkadot&threshold=10&unit=weight&path_pattern=runtime/polkadot/src/weights/*.rs&method=base&ignore_errors=false&git_pull=false";

		let Ok(resp) = reqwest::blocking::get(url).map(|r| r.text().unwrap()) else { continue };

		// Some magic numbers: utility::batch_all and staking::validate old equations
		if !resp.contains("12.68M + 4.41M * c + READ + WRITE") ||
			!resp.contains("41.30M + 12 * READ + 8 * WRITE")
		{
			panic!("Unexpected response: {}", resp);
		} else {
			return
		}
	}
	panic!("Failed to make request in time");
}
