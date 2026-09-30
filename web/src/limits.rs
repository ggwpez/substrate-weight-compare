//! Resource checks performed before opening repositories or binding a listener.

use std::io;

const MIN_OPEN_FILES: libc::rlim_t = 65_536;

pub fn check_open_files() -> io::Result<()> {
	let mut limit = std::mem::MaybeUninit::<libc::rlimit>::uninit();
	// SAFETY: getrlimit writes a complete rlimit to the valid output pointer
	// on success; we only read it after checking the return value.
	if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, limit.as_mut_ptr()) } != 0 {
		return Err(io::Error::other(format!(
			"Cannot check the open-file limit (RLIMIT_NOFILE): {}",
			io::Error::last_os_error(),
		)))
	}
	// SAFETY: getrlimit succeeded and initialized the structure above.
	let limit = unsafe { limit.assume_init() };
	if limit.rlim_cur != libc::RLIM_INFINITY && limit.rlim_cur < MIN_OPEN_FILES {
		return Err(io::Error::other(format!(
			"Open-file soft limit is {} (RLIMIT_NOFILE); subweight-web requires at least {}. \
			 Run `ulimit -Sn {}` in the launching shell, or set `LimitNOFILE={}` in the \
			 systemd service, then restart subweight-web. Raise the hard limit first if necessary.",
			limit.rlim_cur, MIN_OPEN_FILES, MIN_OPEN_FILES, MIN_OPEN_FILES,
		)))
	}
	Ok(())
}
