//! Bounded Git subprocess execution, including its transport helpers.

use std::{
	io::{self, Read},
	process::{Command, Output, Stdio},
	thread,
	time::{Duration, Instant},
};

/// Run Git without interactive prompts, with a two-minute wall-clock deadline.
pub fn git_output(command: &mut Command) -> io::Result<Output> {
	command.env("GIT_TERMINAL_PROMPT", "0");
	output_with_timeout(command, Duration::from_secs(120))
}

fn output_with_timeout(command: &mut Command, timeout: Duration) -> io::Result<Output> {
	#[cfg(unix)]
	{
		use std::os::unix::process::CommandExt;
		command.process_group(0);
	}
	let mut child = command
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;
	let mut stdout = child.stdout.take().expect("piped stdout");
	let mut stderr = child.stderr.take().expect("piped stderr");
	// Drain both pipes concurrently so a full pipe cannot deadlock Git.
	let out = thread::spawn(move || {
		let mut bytes = Vec::new();
		stdout.read_to_end(&mut bytes).map(|_| bytes)
	});
	let err = thread::spawn(move || {
		let mut bytes = Vec::new();
		stderr.read_to_end(&mut bytes).map(|_| bytes)
	});
	let deadline = Instant::now() + timeout;
	let result = loop {
		match child.try_wait() {
			Ok(Some(status)) if out.is_finished() && err.is_finished() => break Ok(status),
			Ok(_) => (),
			Err(e) => break Err(e),
		}
		if Instant::now() >= deadline {
			break Err(io::Error::new(
				io::ErrorKind::TimedOut,
				format!("Git command exceeded {} seconds", timeout.as_secs_f64()),
			));
		}
		thread::sleep(Duration::from_millis(10));
	};
	if result.is_err() {
		// Git launches remote-https/ssh helpers. Killing only Git leaves them
		// alive and may leave the output pipes open indefinitely.
		#[cfg(unix)]
		unsafe {
			libc::kill(-(child.id() as i32), libc::SIGKILL);
		}
		let _ = child.kill();
		let _ = child.wait();
	}
	let stdout = out.join().map_err(|_| io::Error::other("stdout reader panicked"))?;
	let stderr = err.join().map_err(|_| io::Error::other("stderr reader panicked"))?;
	Ok(Output { status: result?, stdout: stdout?, stderr: stderr? })
}

#[cfg(all(test, unix))]
mod tests {
	use super::*;

	#[test]
	fn captures_large_output_and_nonzero_status() {
		let output = output_with_timeout(
			Command::new("sh")
				.args(["-c", "head -c 131072 /dev/zero; head -c 131072 /dev/zero >&2; exit 7"]),
			Duration::from_secs(5),
		)
		.unwrap();
		assert_eq!(output.stdout.len(), 131072);
		assert_eq!(output.stderr.len(), 131072);
		assert_eq!(output.status.code(), Some(7));
	}

	#[test]
	fn deadline_kills_helpers_holding_output_pipes() {
		let start = Instant::now();
		// The parent exits but a helper still holds both pipes open.
		let error = output_with_timeout(
			Command::new("sh").args(["-c", "sleep 30 & exit 0"]),
			Duration::from_millis(100),
		)
		.unwrap_err();
		assert_eq!(error.kind(), io::ErrorKind::TimedOut);
		assert!(start.elapsed() < Duration::from_secs(3));
	}
}
