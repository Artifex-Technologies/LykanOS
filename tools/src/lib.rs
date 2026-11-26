//! LykanOS's own programs, one crate: `lykanos-build` (the ISO), `lykanos-install-cleanup` (the installer's last step inside the
//! new system) and `lykanos-tool-menu` (the security tools' start-menu entries, from a pacman hook); with the `art` feature
//! also `lykanos-art` (the logo and the wallpaper, made on my computer, not shipped). What they share: running
//! commands, finding a program, knowing whether this is root.

#[cfg(feature = "art")]
pub mod art;
pub mod build;
pub mod cleanup;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Runs a command with nothing on its input and its output dropped; whether it ran and succeeded.
pub fn run_quiet(argv: &[&str]) -> bool {
    let Some((prog, args)) = argv.split_first() else {
        return false;
    };
    Command::new(prog)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Runs a command with its output shown (the build's steps); whether it succeeded.
pub fn run_shown(argv: &[&str]) -> bool {
    let Some((prog, args)) = argv.split_first() else {
        return false;
    };
    Command::new(prog)
        .args(args)
        .stdin(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// A program on PATH.
pub fn which(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|p| {
        std::env::split_paths(&p)
            .map(|d| d.join(name))
            .find(|f| is_executable(f))
    })
}

fn is_executable(f: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    f.metadata()
        .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

pub fn is_root() -> bool {
    // SAFETY: geteuid has no preconditions and cannot fail
    unsafe { libc::geteuid() == 0 }
}

/// Copies the folder `from` to `to` (made as needed), files with their modes and links as links.
pub fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let (src, dst) = (e.path(), to.join(e.file_name()));
        let kind = e.file_type()?;
        if kind.is_symlink() {
            let _ = std::fs::remove_file(&dst);
            std::os::unix::fs::symlink(std::fs::read_link(&src)?, &dst)?;
        } else if kind.is_dir() {
            copy_tree(&src, &dst)?;
            std::fs::set_permissions(&dst, std::fs::metadata(&src)?.permissions())?;
        } else {
            std::fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}
