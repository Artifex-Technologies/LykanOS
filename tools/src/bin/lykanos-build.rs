//! lykanos-build: the LykanOS ISO (lykanos_tools::build). As yourself `cargo build --release` in tools/, then as root
//! `tools/target/release/lykanos-build`.

use std::path::PathBuf;

fn main() {
    // the repository this was built from, unless LYKANOS_REPO names another checkout
    let repo = std::env::var_os("LYKANOS_REPO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")));
    let dir = |var: &str, default: &str| {
        std::env::var_os(var)
            .map(PathBuf::from)
            .unwrap_or_else(|| repo.join(default))
    };
    let programs = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    let places = lykanos_tools::build::Places {
        work: dir("LYKANOS_WORK_DIR", "work"),
        out: dir("LYKANOS_OUT_DIR", "out"),
        repo,
        programs,
    };
    if let Err(e) = lykanos_tools::build::build(&places) {
        eprintln!("lykanos-build: error: {e}");
        std::process::exit(1);
    }
}
