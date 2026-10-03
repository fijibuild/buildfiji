//! Sandboxing strategies for local execution.
//!
//! Bazel offers `local`, `sandboxed` (linux-sandbox / darwin-sandbox /
//! processwrapper-sandbox), `worker`, and `docker`. fjfj mirrors that with a
//! `Sandbox` trait so strategies are pluggable and selectable via the Bazel
//! `--spawn_strategy` / `--strategy=Mnemonic=...` flags.

use std::ffi::CString;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// No isolation; run in a scratch execroot.
    Local,
    /// Linux user namespaces + mount namespaces (like `linux-sandbox`).
    LinuxNamespaces,
    /// macOS `sandbox-exec` profile (like `darwin-sandbox`).
    DarwinSeatbelt,
    /// Overlay/hermetic container via OCI runtime.
    Oci,
}

pub trait Sandbox {
    fn strategy(&self) -> Strategy;
    /// Materialise an execroot containing exactly `inputs` at `root`.
    fn prepare(&self, root: &Path) -> anyhow::Result<()>;
}

/// Pick the best available strategy for the host OS.
pub fn default_strategy() -> Strategy {
    if cfg!(target_os = "linux") {
        Strategy::LinuxNamespaces
    } else if cfg!(target_os = "macos") {
        Strategy::DarwinSeatbelt
    } else {
        Strategy::Local
    }
}

/// What a namespace-isolated command may touch (Bazel's `linux-sandbox`).
#[derive(Debug, Clone, Default)]
pub struct Isolation {
    /// Directories that stay writable; the rest of the file system is
    /// read-only.
    pub writable: Vec<PathBuf>,
    /// Give the command a network namespace with nothing in it
    /// (`block-network`).
    pub block_network: bool,
}

/// Make `command` run in new user, mount, pid, ipc and uts namespaces (and a
/// net namespace if asked): the root file system is read-only except for
/// `writable`, `/tmp` and `/dev/shm`, `/proc` is the namespace's own, and the
/// command is killed with its parent.
///
/// The step before `exec` is async-signal-safe: every string is made here,
/// and the child only makes system calls.
pub fn isolate(command: &mut std::process::Command, iso: &Isolation) -> io::Result<()> {
    let mut writable = vec![];
    for dir in iso
        .writable
        .iter()
        .map(PathBuf::as_path)
        .chain([Path::new("/tmp"), Path::new("/dev/shm")])
    {
        if dir.is_dir() {
            writable.push(
                CString::new(dir.as_os_str().as_bytes())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
            );
        }
    }
    let readonly = readonly_mounts(&writable);
    // `std` has gone to the working directory before the step below runs, in
    // the file system as it was; it is entered again once that has changed.
    let cwd = match command.get_current_dir() {
        Some(dir) => Some(
            CString::new(dir.as_os_str().as_bytes())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
        ),
        None => None,
    };
    let mut flags = libc::CLONE_NEWUSER
        | libc::CLONE_NEWNS
        | libc::CLONE_NEWPID
        | libc::CLONE_NEWIPC
        | libc::CLONE_NEWUTS;
    if iso.block_network {
        flags |= libc::CLONE_NEWNET;
    }
    // SAFETY: the closure calls only system calls and touches no allocator or
    // lock.
    unsafe {
        command.pre_exec(move || enter(flags, &writable, &readonly, cwd.as_deref()));
    }
    Ok(())
}

/// The mounts to make read-only, each with the flags it has now (a remount
/// that drops `nosuid` and the like is refused): everything in
/// `/proc/self/mountinfo` but `/proc`, `/sys`, `/dev` and the writable paths.
fn readonly_mounts(writable: &[CString]) -> Vec<(CString, libc::c_ulong)> {
    let Ok(info) = std::fs::read_to_string("/proc/self/mountinfo") else {
        return vec![(CString::from(c"/"), 0)];
    };
    let mut mounts = vec![];
    for line in info.lines() {
        let Some(at) = line.split(' ').nth(4) else {
            continue;
        };
        let at = unescape(at);
        let skip = ["/proc", "/sys", "/dev"]
            .iter()
            .any(|p| at == p.as_bytes() || at.starts_with(format!("{p}/").as_bytes()));
        let Ok(at) = CString::new(at) else {
            continue;
        };
        if skip || writable.contains(&at) {
            continue;
        }
        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
        if unsafe { libc::statvfs(at.as_ptr(), &mut stat) } != 0 {
            continue;
        }
        let mut kept = 0;
        for (st, ms) in [
            (libc::ST_NOSUID, libc::MS_NOSUID),
            (libc::ST_NODEV, libc::MS_NODEV),
            (libc::ST_NOEXEC, libc::MS_NOEXEC),
            (libc::ST_NOATIME, libc::MS_NOATIME),
            (libc::ST_NODIRATIME, libc::MS_NODIRATIME),
            (libc::ST_RELATIME, libc::MS_RELATIME),
        ] {
            if stat.f_flag & st != 0 {
                kept |= ms;
            }
        }
        mounts.push((at, kept));
    }
    if mounts.is_empty() {
        mounts.push((CString::from(c"/"), 0));
    }
    mounts
}

/// A mountinfo path: spaces and the like are written as `\040`.
fn unescape(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let (mut out, mut i) = (vec![], 0);
    while i < b.len() {
        let octal = b
            .get(i + 1..i + 4)
            .filter(|d| d.iter().all(|c| (b'0'..=b'7').contains(c)));
        match (b[i], octal) {
            (b'\\', Some(d)) => {
                out.push((d[0] - b'0') << 6 | (d[1] - b'0') << 3 | (d[2] - b'0'));
                i += 4;
            }
            (c, _) => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

fn fail<T>() -> io::Result<T> {
    Err(io::Error::last_os_error())
}

/// Write `value` to the file at `path` (a NUL-terminated byte string).
unsafe fn put(path: &[u8], value: &[u8]) -> io::Result<()> {
    unsafe {
        let fd = libc::open(path.as_ptr().cast(), libc::O_WRONLY);
        if fd < 0 {
            return fail();
        }
        let n = libc::write(fd, value.as_ptr().cast(), value.len());
        libc::close(fd);
        if n < 0 { fail() } else { Ok(()) }
    }
}

/// `"<id> <id> 1\n"` in `buf`, the length.
fn id_map(id: u32, buf: &mut [u8; 32]) -> usize {
    let mut digits = [0u8; 10];
    let (mut n, mut at) = (id, digits.len());
    loop {
        at -= 1;
        digits[at] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    let digits = &digits[at..];
    let mut len = 0;
    for part in [digits, b" ", digits, b" 1\n"] {
        buf[len..len + part.len()].copy_from_slice(part);
        len += part.len();
    }
    len
}

fn enter(
    flags: libc::c_int,
    writable: &[CString],
    readonly: &[(CString, libc::c_ulong)],
    cwd: Option<&std::ffi::CStr>,
) -> io::Result<()> {
    unsafe {
        let (uid, gid) = (libc::getuid(), libc::getgid());
        if libc::unshare(flags) != 0 {
            return fail();
        }
        let mut buf = [0u8; 32];
        put(b"/proc/self/setgroups\0", b"deny")?;
        let n = id_map(uid, &mut buf);
        put(b"/proc/self/uid_map\0", &buf[..n])?;
        let n = id_map(gid, &mut buf);
        put(b"/proc/self/gid_map\0", &buf[..n])?;
        // The first child of a new pid namespace is its init; this process
        // stays outside, passes on how it ended, and the command is the init.
        let child = libc::fork();
        if child < 0 {
            return fail();
        }
        if child > 0 {
            let mut status = 0;
            while libc::waitpid(child, &mut status, 0) < 0 {
                if *libc::__errno_location() != libc::EINTR {
                    libc::_exit(127);
                }
            }
            if libc::WIFSIGNALED(status) {
                let sig = libc::WTERMSIG(status);
                libc::signal(sig, libc::SIG_DFL);
                libc::kill(libc::getpid(), sig);
                libc::_exit(128 + sig);
            }
            libc::_exit(libc::WEXITSTATUS(status));
        }
        libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
        if libc::getppid() == 1 {
            libc::_exit(127);
        }
        let none = std::ptr::null();
        if libc::mount(
            none,
            c"/".as_ptr(),
            none,
            libc::MS_REC | libc::MS_PRIVATE,
            none.cast(),
        ) != 0
        {
            return fail();
        }
        // Best effort: without it `/proc` shows the parent namespace.
        libc::mount(
            c"proc".as_ptr(),
            c"/proc".as_ptr(),
            c"proc".as_ptr(),
            libc::MS_NOSUID | libc::MS_NODEV | libc::MS_NOEXEC,
            none.cast(),
        );
        for dir in writable {
            if libc::mount(
                dir.as_ptr(),
                dir.as_ptr(),
                none,
                libc::MS_BIND | libc::MS_REC,
                none.cast(),
            ) != 0
            {
                return fail();
            }
        }
        // Each mount is its own file system as far as a remount goes, so `/`
        // alone would leave `/home` and the like writable.
        for (at, kept) in readonly {
            let done = libc::mount(
                none,
                at.as_ptr(),
                none,
                libc::MS_REMOUNT | libc::MS_BIND | libc::MS_RDONLY | kept,
                none.cast(),
            );
            if done != 0 && at.as_bytes() == b"/" {
                return fail();
            }
        }
        if let Some(cwd) = cwd
            && libc::chdir(cwd.as_ptr()) != 0
        {
            return fail();
        }
    }
    Ok(())
}

/// Whether this host lets a command run in namespaces (user namespaces may be
/// off, or the process may be in a container that forbids them).
pub fn namespaces_available() -> bool {
    static AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        let mut probe = std::process::Command::new("/bin/true");
        isolate(&mut probe, &Isolation::default()).is_ok()
            && probe.status().is_ok_and(|s| s.success())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(script: &str, iso: &Isolation) -> std::process::Output {
        let mut c = std::process::Command::new("/bin/sh");
        c.args(["-c", script]);
        isolate(&mut c, iso).unwrap();
        c.output().unwrap()
    }

    #[test]
    fn the_file_system_is_read_only_outside_the_writable_paths() {
        if !namespaces_available() {
            return;
        }
        // A directory we own, so only the mount stops a write.
        let owned = std::env::current_dir().unwrap();
        if owned.starts_with("/tmp") || owned.starts_with("/dev/shm") {
            return;
        }
        let probe = owned.join("fjfj-sandbox-probe");
        let out = run(
            &format!("! touch {}", probe.display()),
            &Isolation::default(),
        );
        assert!(out.status.success() && !probe.exists(), "{:?}", out);
        let out = run(
            &format!("touch {}", probe.display()),
            &Isolation {
                writable: vec![owned.clone()],
                ..Isolation::default()
            },
        );
        assert!(out.status.success() && probe.exists(), "{:?}", out);
        let _ = std::fs::remove_file(&probe);
    }

    #[test]
    fn the_command_is_pid_1_and_its_exit_status_comes_out() {
        if !namespaces_available() {
            return;
        }
        let out = run("test $$ = 1 && exit 7", &Isolation::default());
        assert_eq!(out.status.code(), Some(7));
    }

    #[test]
    fn a_blocked_network_has_only_loopback() {
        if !namespaces_available() {
            return;
        }
        let iso = Isolation {
            block_network: true,
            ..Isolation::default()
        };
        // Two header lines and `lo`.
        let out = run("test $(wc -l < /proc/net/dev) -le 3", &iso);
        assert!(out.status.success(), "{:?}", out);
    }
}
