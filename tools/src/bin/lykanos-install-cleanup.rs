//! lykanos-install-cleanup: run by the installer (yggdrasil-install-system, named in the image's
//! /usr/share/yggdrasil/integrations/lykanos-installer.conf) inside the new system, right after the live image was copied onto
//! the disk. What it does and why: lykanos_tools::cleanup.

fn main() {
    let mut run = |argv: &[&str]| lykanos_tools::run_quiet(argv);
    let as_root = lykanos_tools::is_root();
    match lykanos_tools::cleanup::cleanup(std::path::Path::new("/"), &mut run, as_root) {
        Ok(said) => said
            .iter()
            .for_each(|s| println!("lykanos-install-cleanup: {s}")),
        Err((said, e)) => {
            said.iter()
                .for_each(|s| println!("lykanos-install-cleanup: {s}"));
            eprintln!("lykanos-install-cleanup: error: {e}");
            std::process::exit(1);
        }
    }
}
