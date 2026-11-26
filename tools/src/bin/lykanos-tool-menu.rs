//! lykanos-tool-menu: start-menu entries for LykanOS's security tools, which mostly ship none (13 of 15 sampled on the
//! 2026-09-12 ISO had no entry, so the desktop showed no sign of them). Sarina's `desktop-audit` proposes an entry for each
//! installed command-line tool from its pacman group and description; only those with the Security category are written,
//! system-wide, in /usr/local/share/applications (Yggdrasil's start menu lists them under Security and opens them in a
//! terminal). Run by a pacman hook after every transaction (also while the ISO is built), so a tool installed later gets an
//! entry and a removed one loses it. Only entries Sarina marked as hers (X-Sarina-Generated) are ever replaced or removed.
//! It never fails a pacman transaction: without Sarina, or when a step fails, it says nothing and ends well.

const DIR: &str = "/usr/local/share/applications";

fn main() {
    if lykanos_tools::which("sarina").is_none() || std::fs::create_dir_all(DIR).is_err() {
        return;
    }
    lykanos_tools::run_quiet(&[
        "sarina",
        "desktop-audit",
        "--remove-generated",
        "--target-dir",
        DIR,
    ]);
    lykanos_tools::run_quiet(&[
        "sarina",
        "desktop-audit",
        "--apply",
        "--only-category",
        "Security",
        "--target-dir",
        DIR,
    ]);
}
