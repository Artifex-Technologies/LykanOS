//! lykanos-art: LykanOS's logo set and wallpaper (lykanos_tools::art), made again from assets/logo.
//!
//!     cargo run --release --features art --bin lykanos-art -- logo
//!     cargo run --release --features art --bin lykanos-art -- wallpaper [OUT.png [WIDTH HEIGHT]]
//!
//! LYKANOS_REPO names another checkout to write into.

use std::path::PathBuf;

fn main() {
    let repo = std::env::var_os("LYKANOS_REPO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")));
    let args: Vec<String> = std::env::args().skip(1).collect();
    let done = match args.first().map(String::as_str) {
        Some("logo") => lykanos_tools::art::logo(&repo),
        Some("wallpaper") => {
            let out = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
                repo.join("profile/airootfs/usr/share/wallpapers/lykanos/lykanos.png")
            });
            let size = |i: usize, d: u32| args.get(i).and_then(|v| v.parse().ok()).unwrap_or(d);
            lykanos_tools::art::wallpaper_to(&out, size(2, 2560), size(3, 1440))
        }
        _ => Err("usage: lykanos-art logo | wallpaper [OUT.png [WIDTH HEIGHT]]".into()),
    };
    if let Err(e) = done {
        eprintln!("lykanos-art: {e}");
        std::process::exit(1);
    }
}
