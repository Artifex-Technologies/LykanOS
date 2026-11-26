//! The installer's last step, inside the new system right after the live image was copied onto the disk: it takes away what
//! only the live medium needs and puts back what mkarchiso strips, so the installed system boots on its own and has a real
//! login. Everything here exists because the live image is not an installable root:
//!
//!  * mkarchiso empties /boot, so there is no kernel to boot;
//!  * the start-up image's preset and drop-in are archiso's, building an image that looks for the live squashfs and cannot
//!    mount a real root;
//!  * the live account is passwordless: sudo, polkit and the login screen's automatic sign-in (greetd's live.toml) were all
//!    opened up for it.
//!
//! Keep LIVE_ONLY in step with profile/airootfs/etc.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// The files only the live medium has.
pub const LIVE_ONLY: [&str; 9] = [
    "/etc/mkinitcpio.conf.d/archiso.conf",
    "/etc/greetd/live.toml",
    "/etc/systemd/system/greetd.service.d/live.conf",
    "/etc/sudoers.d/00-live-nopasswd",
    "/etc/polkit-1/rules.d/49-live-nopasswd.rules",
    "/etc/sysusers.d/live.conf",
    "/etc/tmpfiles.d/live-home.conf",
    "/usr/share/applications/lykanos-install.desktop",
    "/usr/share/yggdrasil/integrations/lykanos-installer.conf",
];

/// The keys pacman needs on the installed system: Arch's, BlackArch's (the tools), CachyOS's (the kernel) and Labyrinth's
/// (the desktop, Sarina and Heimdall come from its signed repository).
pub const KEYRINGS: [&str; 4] = ["archlinux", "blackarch", "cachyos", "labyrinth"];

/// Messages said so far, and what went wrong.
pub type Failure = (Vec<String>, String);

/// The clean-up of the system at `root` (`/` in the new system; a made-up tree in tests). `run` runs a command there
/// (pacman-key, systemctl); `as_root`: the seeded settings are given to root (a test cannot).
pub fn cleanup(
    root: &Path,
    run: &mut dyn FnMut(&[&str]) -> bool,
    as_root: bool,
) -> Result<Vec<String>, Failure> {
    let mut said = vec![];
    let at = |p: &str| root.join(p.trim_start_matches('/'));
    macro_rules! fail {
        ($($t:tt)*) => { return Err((said, format!($($t)*))) };
    }

    // 1. New accounts start with the live user's desktop settings, if it has any (Yggdrasil writes its own at the first
    //    sign-in). /etc/skel is what useradd copies from, so the installer's useradd picks them up and handles ownership.
    let live_config = at("/home/live/.config");
    if live_config.is_dir() {
        let skel = at("/etc/skel/.config");
        if let Err(e) = crate::copy_tree(&live_config, &skel) {
            fail!("could not seed /etc/skel from the live user's .config: {e}");
        }
        // the copy kept the live user's uid (1000), the first installed account's: the template every new account starts
        // from belongs to root and is readable by all
        if let Err(e) = give_to_root(&skel, as_root) {
            fail!("could not give /etc/skel/.config to root: {e}");
        }
        said.push("seeded /etc/skel from the live user's .config".into());
    }

    // 2. The live-only files and the live account's home.
    for f in LIVE_ONLY {
        if let Err(e) = std::fs::remove_file(at(f)) {
            if e.kind() != std::io::ErrorKind::NotFound {
                fail!("could not remove {f}: {e}");
            }
        }
    }
    let _ = std::fs::remove_dir(at("/etc/systemd/system/greetd.service.d"));
    if let Err(e) = std::fs::remove_dir_all(at("/home/live")) {
        if e.kind() != std::io::ErrorKind::NotFound {
            fail!("could not remove /home/live: {e}");
        }
    }
    said.push("removed live-only configuration".into());

    // 3 and 4. Each installed kernel (its package base in /usr/lib/modules/*/pkgbase): the stock start-up image preset, made
    //    from mkinitcpio's own template as pacman's hook makes it (not a hand-written copy that could drift; it builds only the
    //    default image, which is also quicker), and the kernel back in /boot, from the package's own copy under
    //    /usr/lib/modules, which mkarchiso left.
    let template =
        std::fs::read_to_string(at("/usr/share/mkinitcpio/hook.preset")).unwrap_or_default();
    if template.trim().is_empty() {
        fail!("/usr/share/mkinitcpio/hook.preset is missing or empty, cannot restore the presets");
    }
    let kernels = kernels(&at("/usr/lib/modules"));
    if kernels.is_empty() {
        fail!("no kernel found under /usr/lib/modules");
    }
    if let Err(e) = std::fs::create_dir_all(at("/etc/mkinitcpio.d")) {
        fail!("could not make /etc/mkinitcpio.d: {e}");
    }
    for (base, modules) in &kernels {
        let preset = template.replace("%PKGBASE%", base);
        // an empty or wrong preset builds a start-up image that cannot boot, and nothing would notice before the first start
        if !preset.lines().any(|l| l.starts_with("PRESETS=")) {
            fail!("the preset made for {base} has no PRESETS line");
        }
        let target = at(&format!("/etc/mkinitcpio.d/{base}.preset"));
        if let Err(e) = std::fs::write(&target, preset) {
            fail!("could not write {}: {e}", target.display());
        }
        let vmlinuz = at(&format!("/boot/vmlinuz-{base}"));
        let copied = std::fs::create_dir_all(at("/boot"))
            .and_then(|_| std::fs::copy(modules.join("vmlinuz"), &vmlinuz))
            .and_then(|_| {
                std::fs::set_permissions(&vmlinuz, std::fs::Permissions::from_mode(0o644))
            });
        if let Err(e) = copied {
            fail!("could not put {base}'s kernel in /boot: {e}");
        }
        said.push(format!(
            "restored /boot/vmlinuz-{base} and its stock mkinitcpio preset"
        ));
    }

    // 5. pacman can check packages: the installed system needs it for `pacman -Syu` and for Sarina to install anything
    //    later. A keyring is made only when the image carried none; the keys are put in either way.
    let ring = at("/etc/pacman.d/gnupg/pubring.gpg");
    if std::fs::metadata(&ring).map(|m| m.len()).unwrap_or(0) == 0 {
        said.push("no pacman keyring in the image; initialising".into());
        if !run(&["pacman-key", "--init"]) {
            fail!("pacman-key --init failed");
        }
    }
    let mut populate = vec!["pacman-key", "--populate"];
    populate.extend(KEYRINGS);
    if !run(&populate) {
        fail!("pacman-key --populate {} failed", KEYRINGS.join(" "));
    }

    // 6. Heimdall is installed but off (/etc/heimdall/defaults-off: its install hook switched nothing on, until it has rules
    //    for each BlackArch tool). ClamAV's signature updates are on. The firewall (ufw) is installed but off too: listeners
    //    for the security tools need incoming connections; `sudo ufw enable` turns it on.
    if at("/usr/lib/systemd/system/clamav-freshclam.service").exists() {
        said.push(
            if run(&["systemctl", "enable", "clamav-freshclam.service"]) {
                "enabled ClamAV signature updates".into()
            } else {
                "warning: could not enable ClamAV signature updates".into()
            },
        );
    }
    Ok(said)
}

/// Each kernel under `modules` (/usr/lib/modules) with a package base and a vmlinuz: (package base, its folder), by base.
fn kernels(modules: &Path) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = std::fs::read_dir(modules)
        .map(|r| {
            r.flatten()
                .filter_map(|e| {
                    let dir = e.path();
                    let base = std::fs::read_to_string(dir.join("pkgbase"))
                        .ok()?
                        .trim()
                        .to_string();
                    let ok = !base.is_empty()
                        && base.chars().all(|c| {
                            c.is_ascii_lowercase()
                                || c.is_ascii_digit()
                                || matches!(c, '-' | '_' | '.')
                        })
                        && !base.starts_with('.');
                    (ok && dir.join("vmlinuz").is_file()).then_some((base, dir))
                })
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out.dedup_by(|a, b| a.0 == b.0);
    out
}

/// Everything under `dir` to uid and gid 0 (when `as_root`), and modes as `chmod -R u+rwX,go+rX,go-w`: the owner reads and
/// writes, everyone reads, folders and programs can be entered and run, nobody else writes.
fn give_to_root(dir: &Path, as_root: bool) -> std::io::Result<()> {
    let meta = std::fs::symlink_metadata(dir)?;
    if as_root {
        std::os::unix::fs::lchown(dir, Some(0), Some(0))?;
    }
    if meta.file_type().is_symlink() {
        return Ok(());
    }
    let mut mode = meta.permissions().mode() & 0o7777;
    let x = meta.is_dir() || mode & 0o111 != 0;
    mode |= 0o600 | 0o044;
    if x {
        mode |= 0o111;
    }
    mode &= !0o022;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(mode))?;
    if meta.is_dir() {
        for e in std::fs::read_dir(dir)? {
            give_to_root(&e?.path(), as_root)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Root(PathBuf);

    impl Root {
        fn new(tag: &str) -> Root {
            let d =
                std::env::temp_dir().join(format!("lykanos-cleanup-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&d);
            std::fs::create_dir_all(&d).unwrap();
            Root(d)
        }

        fn put(&self, p: &str, data: &str) {
            let f = self.0.join(p.trim_start_matches('/'));
            std::fs::create_dir_all(f.parent().unwrap()).unwrap();
            std::fs::write(f, data).unwrap();
        }

        fn has(&self, p: &str) -> bool {
            self.0.join(p.trim_start_matches('/')).exists()
        }

        fn read(&self, p: &str) -> String {
            std::fs::read_to_string(self.0.join(p.trim_start_matches('/'))).unwrap()
        }
    }

    impl Drop for Root {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const TEMPLATE: &str = "ALL_kver=\"/boot/vmlinuz-%PKGBASE%\"\nPRESETS=('default')\ndefault_image=\"/boot/initramfs-%PKGBASE%.img\"\n";

    /// A copied live image: live-only files, the live user's settings, two kernels without /boot, no keyring.
    fn copied(tag: &str) -> Root {
        let r = Root::new(tag);
        for f in LIVE_ONLY {
            r.put(f, "live");
        }
        r.put("/home/live/.config/yggdrasil/panel.conf", "dock = left\n");
        r.put("/usr/share/mkinitcpio/hook.preset", TEMPLATE);
        r.put(
            "/usr/lib/modules/7.3.5-2-cachyos/pkgbase",
            "linux-cachyos\n",
        );
        r.put("/usr/lib/modules/7.3.5-2-cachyos/vmlinuz", "kernel A");
        r.put(
            "/usr/lib/modules/6.18.9-1-lts/pkgbase",
            "linux-cachyos-lts\n",
        );
        r.put("/usr/lib/modules/6.18.9-1-lts/vmlinuz", "kernel B");
        r.put("/usr/lib/modules/extramodules-7.3/version", "7.3.5\n");
        r.put(
            "/usr/lib/systemd/system/clamav-freshclam.service",
            "[Unit]\n",
        );
        r
    }

    #[test]
    fn the_live_image_becomes_an_installed_system() {
        let r = copied("full");
        let mut calls: Vec<String> = vec![];
        let said = cleanup(
            &r.0,
            &mut |a| {
                calls.push(a.join(" "));
                true
            },
            false,
        )
        .unwrap();
        for f in LIVE_ONLY {
            assert!(!r.has(f), "{f} left");
        }
        assert!(!r.has("/home/live") && !r.has("/etc/systemd/system/greetd.service.d"));
        assert_eq!(
            r.read("/etc/skel/.config/yggdrasil/panel.conf"),
            "dock = left\n"
        );
        let mode = std::fs::metadata(r.0.join("etc/skel/.config/yggdrasil"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode & 0o755, 0o755, "the template's folders open to all");
        assert_eq!(mode & 0o022, 0, "and written by none but the owner");
        for (base, k) in [
            ("linux-cachyos", "kernel A"),
            ("linux-cachyos-lts", "kernel B"),
        ] {
            assert_eq!(r.read(&format!("/boot/vmlinuz-{base}")), k);
            assert_eq!(
                r.read(&format!("/etc/mkinitcpio.d/{base}.preset")),
                TEMPLATE.replace("%PKGBASE%", base)
            );
        }
        assert_eq!(
            calls,
            [
                "pacman-key --init",
                "pacman-key --populate archlinux blackarch cachyos labyrinth",
                "systemctl enable clamav-freshclam.service"
            ]
        );
        assert_eq!(
            said.last().map(String::as_str),
            Some("enabled ClamAV signature updates")
        );
    }

    #[test]
    fn a_keyring_in_the_image_is_kept_and_a_missing_live_config_is_no_error() {
        let r = copied("ring");
        r.put("/etc/pacman.d/gnupg/pubring.gpg", "keys");
        std::fs::remove_dir_all(r.0.join("home/live/.config")).unwrap();
        let mut calls: Vec<String> = vec![];
        cleanup(
            &r.0,
            &mut |a| {
                calls.push(a.join(" "));
                true
            },
            false,
        )
        .unwrap();
        assert_eq!(
            calls[0], "pacman-key --populate archlinux blackarch cachyos labyrinth",
            "no --init over a keyring"
        );
        assert!(!r.has("/etc/skel/.config"));
    }

    #[test]
    fn it_stops_without_a_kernel_a_template_or_the_keys() {
        let r = copied("nokernel");
        std::fs::remove_dir_all(r.0.join("usr/lib/modules")).unwrap();
        let (_, e) = cleanup(&r.0, &mut |_| true, false).unwrap_err();
        assert_eq!(e, "no kernel found under /usr/lib/modules");

        let r = copied("notemplate");
        r.put("/usr/share/mkinitcpio/hook.preset", "  \n");
        assert!(cleanup(&r.0, &mut |_| true, false)
            .unwrap_err()
            .1
            .contains("hook.preset is missing or empty"));

        let r = copied("badtemplate");
        r.put("/usr/share/mkinitcpio/hook.preset", "nothing here\n");
        assert_eq!(
            cleanup(&r.0, &mut |_| true, false).unwrap_err().1,
            "the preset made for linux-cachyos has no PRESETS line"
        );

        let r = copied("nokeys");
        let (said, e) = cleanup(&r.0, &mut |a| a[1] != "--populate", false).unwrap_err();
        assert_eq!(
            e,
            "pacman-key --populate archlinux blackarch cachyos labyrinth failed"
        );
        assert!(
            said.iter()
                .any(|s| s.starts_with("restored /boot/vmlinuz-")),
            "what was done is still said"
        );
    }

    #[test]
    fn a_kernel_with_an_odd_package_base_is_not_used() {
        let r = copied("odd");
        r.put("/usr/lib/modules/x/pkgbase", "../../etc/evil\n");
        r.put("/usr/lib/modules/x/vmlinuz", "x");
        cleanup(&r.0, &mut |_| true, false).unwrap();
        assert!(!r.has("/etc/evil.preset") && !r.has("/boot/vmlinuz-../../etc/evil"));
    }
}
