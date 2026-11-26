//! The LykanOS ISO. Runs on Arch Linux (bare metal, a VM or an archlinux container) as root (mkarchiso requires it), after
//! `cargo build --release` in tools/ as a normal user (this program puts that build's other programs into the image):
//!
//!   1. BlackArch's signing key and mirror list on this build host, through BlackArch's own strap.sh, unless present;
//!   2. Labyrinth OS's package signing key trusted on this build host by its full fingerprint: the desktop (Yggdrasil), Sarina
//!      and Heimdall come from Labyrinth's signed repository ([labyrinth] in profile/pacman.conf);
//!   3. a work copy of profile/ with the service links made there and LykanOS's own programs (lykanos-install-cleanup,
//!      lykanos-tool-menu) put in its /usr/local/bin;
//!   4. mkarchiso on that copy.
//!
//! Output: out/*.iso (or $LYKANOS_OUT_DIR). LYKANOS_WORK_DIR and LYKANOS_OUT_DIR move work/ and out/ to another file system
//! (a full BlackArch and desktop pacstrap needs far more room than a CI runner's root disk has; CI points them at its scratch
//! disk).

use std::path::{Path, PathBuf};

/// Labyrinth's package signing key, trusted only by this full fingerprint.
pub const LABYRINTH_FPR: &str = "C9CE784C8BD9B661042075972D3BD4B1E7CD07A5";
const LABYRINTH_KEY_URL: &str =
    "https://raw.githubusercontent.com/Artifex-Technologies/Labyrinth_OS/main/packages/pkgbuilds/labyrinth-keyring/labyrinth.gpg";
const BLACKARCH_STRAP: &str = "https://blackarch.org/strap.sh";

/// LykanOS's own programs the image carries in /usr/local/bin, from this crate's build.
pub const PROGRAMS: [&str; 2] = ["lykanos-install-cleanup", "lykanos-tool-menu"];

/// The systemd links made in the work copy, as (link under airootfs/etc/systemd/system, target). They are made here, not
/// kept in git: mkarchiso copies airootfs over the pacstrapped root as it is, and a checkout without symlinks
/// (core.symlinks=false, as this repository's first checkout was) turns a committed link into a text file holding its
/// target, which systemd reads as a broken unit (until 2026-11-07 the login screen, the network and the default target were
/// such files).
pub const LINKS: [(&str, &str); 6] = [
    ("default.target", "/usr/lib/systemd/system/graphical.target"),
    // the login screen: greetd with Yggdrasil's greeter (on the live medium it signs the live user in, greetd.service.d/live.conf)
    (
        "display-manager.service",
        "/usr/lib/systemd/system/greetd.service",
    ),
    (
        "multi-user.target.wants/NetworkManager.service",
        "/usr/lib/systemd/system/NetworkManager.service",
    ),
    (
        "bluetooth.target.wants/bluetooth.service",
        "/usr/lib/systemd/system/bluetooth.service",
    ),
    (
        "multi-user.target.wants/pacman-init.service",
        "/etc/systemd/system/pacman-init.service",
    ),
    ("systemd-firstboot.service", "/dev/null"),
];

/// Where things are: the repository, the work and output folders, and the folder this crate's programs were built in.
pub struct Places {
    pub repo: PathBuf,
    pub work: PathBuf,
    pub out: PathBuf,
    pub programs: PathBuf,
}

/// The work copy of the profile: profile/ copied, the service links made, LykanOS's programs put in /usr/local/bin.
pub fn work_profile(p: &Places) -> Result<PathBuf, String> {
    let dir = p.work.join("profile");
    let _ = std::fs::remove_dir_all(&dir);
    crate::copy_tree(&p.repo.join("profile"), &dir)
        .map_err(|e| format!("could not copy profile/ to {}: {e}", dir.display()))?;
    let sysd = dir.join("airootfs/etc/systemd/system");
    for (link, target) in LINKS {
        let at = sysd.join(link);
        let made = std::fs::create_dir_all(at.parent().unwrap_or(&sysd)).and_then(|_| {
            let _ = std::fs::remove_file(&at);
            std::os::unix::fs::symlink(target, &at)
        });
        made.map_err(|e| format!("could not link {}: {e}", at.display()))?;
    }
    let bin = dir.join("airootfs/usr/local/bin");
    std::fs::create_dir_all(&bin).map_err(|e| format!("could not make {}: {e}", bin.display()))?;
    for prog in PROGRAMS {
        let built = p.programs.join(prog);
        if !built.is_file() {
            return Err(format!("{} is not built: run `cargo build --release` in tools/ first (as yourself, not root)", built.display()));
        }
        std::fs::copy(&built, bin.join(prog))
            .map_err(|e| format!("could not put {prog} in the image: {e}"))?;
    }
    Ok(dir)
}

/// The whole build: keys on this host, the work profile, mkarchiso.
pub fn build(p: &Places) -> Result<(), String> {
    if !crate::is_root() {
        return Err(
            "run as root (mkarchiso requires it), e.g.: sudo tools/target/release/lykanos-build"
                .into(),
        );
    }
    if crate::which("mkarchiso").is_none() {
        return Err("mkarchiso not found: install archiso (pacman -S --needed archiso)".into());
    }
    if Path::new("/etc/pacman.d/blackarch-mirrorlist").exists() {
        println!("==> BlackArch mirror list already present, skipping strap.sh");
    } else {
        println!("==> Installing BlackArch's keyring and mirror list on this build host");
        let strap = std::env::temp_dir().join("blackarch-strap.sh");
        let strap_s = strap.to_string_lossy();
        if !crate::run_shown(&["curl", "-fsSL", BLACKARCH_STRAP, "-o", &strap_s])
            || !crate::run_shown(&["bash", &strap_s])
        {
            return Err("BlackArch's strap.sh failed".into());
        }
    }
    println!("==> Labyrinth's package signing key on the build host");
    if crate::run_quiet(&["pacman-key", "--list-keys", LABYRINTH_FPR]) {
        println!("    already present");
    } else {
        let key = std::env::temp_dir().join(format!("labyrinth-{}.gpg", std::process::id()));
        let key_s = key.to_string_lossy();
        let fetched = crate::run_shown(&["curl", "-fsSL", LABYRINTH_KEY_URL, "-o", &key_s])
            && crate::run_shown(&["pacman-key", "--add", &key_s]);
        let _ = std::fs::remove_file(&key);
        if !fetched {
            return Err("could not fetch and add Labyrinth's key".into());
        }
        let finger = std::process::Command::new("pacman-key")
            .args(["--finger", LABYRINTH_FPR])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).replace(' ', ""))
            .unwrap_or_default();
        if !finger.contains(LABYRINTH_FPR) {
            return Err(format!(
                "the Labyrinth key fetched does not have fingerprint {LABYRINTH_FPR}"
            ));
        }
        if !crate::run_shown(&["pacman-key", "--lsign-key", LABYRINTH_FPR]) {
            return Err("could not sign Labyrinth's key locally".into());
        }
    }
    println!("==> Preparing the work copy of profile/");
    let profile = work_profile(p)?;
    println!("==> Running mkarchiso");
    std::fs::create_dir_all(&p.out)
        .map_err(|e| format!("could not make {}: {e}", p.out.display()))?;
    let (iso_work, out, prof) = (
        p.work.join("iso"),
        p.out.to_string_lossy().into_owned(),
        profile.to_string_lossy().into_owned(),
    );
    if !crate::run_shown(&[
        "mkarchiso",
        "-v",
        "-w",
        &iso_work.to_string_lossy(),
        "-o",
        &out,
        &prof,
    ]) {
        return Err("mkarchiso failed".into());
    }
    println!("==> Done: {out}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_work_profile_has_the_links_and_the_programs() {
        let base = std::env::temp_dir().join(format!("lykanos-build-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let repo = base.join("repo");
        std::fs::create_dir_all(repo.join("profile/airootfs/etc")).unwrap();
        std::fs::write(repo.join("profile/profiledef.sh"), "iso_name=\"lykanos\"\n").unwrap();
        std::fs::write(repo.join("profile/airootfs/etc/hostname"), "lykanos\n").unwrap();
        let programs = base.join("target");
        std::fs::create_dir_all(&programs).unwrap();
        let places = Places {
            repo: repo.clone(),
            work: base.join("work"),
            out: base.join("out"),
            programs: programs.clone(),
        };
        let missing = work_profile(&places).unwrap_err();
        assert!(
            missing.contains("lykanos-install-cleanup is not built"),
            "{missing}"
        );
        for prog in PROGRAMS {
            std::fs::write(programs.join(prog), format!("binary {prog}")).unwrap();
        }
        let dir = work_profile(&places).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.join("profiledef.sh")).unwrap(),
            "iso_name=\"lykanos\"\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("airootfs/etc/hostname")).unwrap(),
            "lykanos\n"
        );
        for (link, target) in LINKS {
            assert_eq!(
                std::fs::read_link(dir.join("airootfs/etc/systemd/system").join(link)).unwrap(),
                Path::new(target)
            );
        }
        for prog in PROGRAMS {
            assert_eq!(
                std::fs::read_to_string(dir.join("airootfs/usr/local/bin").join(prog)).unwrap(),
                format!("binary {prog}")
            );
        }
        // made again over an old copy: the same
        assert_eq!(work_profile(&places).unwrap(), dir);
        let _ = std::fs::remove_dir_all(&base);
    }
}
