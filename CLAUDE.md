# CLAUDE.md — LykanOS

## 1. What this is

LykanOS (named Chymaera OS until 2026-11-07) is an Arch-based security distribution. This repository is an
[archiso](https://github.com/archlinux/archiso) profile (`profile/`), LykanOS's own programs in Rust (`tools/`: `lykanos-build`
turns the profile into a bootable ISO, `lykanos-install-cleanup` is the installer's last step in the new system,
`lykanos-tool-menu` writes the tools' start-menu entries; `lykanos-art`, built only with the `art` feature, makes its logo
set and wallpaper). The desktop
(Yggdrasil on Hyprland, greetd, Yggdrasil's installer), Sarina, Heimdall and `labyrinth-keyring` are packages from Labyrinth
OS's signed repository `[labyrinth]`; their code lives in those repositories. Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
before design decisions; [docs/DESKTOP.md](docs/DESKTOP.md) covers the desktop and what LykanOS adds,
[docs/INSTALLER.md](docs/INSTALLER.md) the install flow, [docs/TOOLS.md](docs/TOOLS.md) the `[lykanos]` repository.

## 2. Build and test

Two steps: LykanOS's programs as yourself, then the ISO as root on an Arch host or container (`mkarchiso` requires both):

```bash
(cd tools && cargo build --release --locked)
sudo tools/target/release/lykanos-build
cd tools && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features   # the checks
cargo run --release --features art --bin lykanos-art -- logo        # in tools/: the logo set again (wallpaper: `-- wallpaper`)
```

`lykanos-build` installs BlackArch's key (BlackArch's `strap.sh`), trusts the Labyrinth package key by its full
fingerprint (`C9CE784C8BD9B661042075972D3BD4B1E7CD07A5`: fetched, fingerprint checked, then `pacman-key --lsign-key`), does
the same for CachyOS's key (`882DCFE48E2051D48E2562ABF3B607488DB35A47`, from keyserver.ubuntu.com) and installs CachyOS's
keyring and mirror list unless the host has its mirror list (the kernel is CachyOS's, `[cachyos]`), makes the systemd
service links in a work copy of the profile, puts the two programs the image carries in its `/usr/local/bin` and
runs `mkarchiso`. `LYKANOS_WORK_DIR` and `LYKANOS_OUT_DIR` move
`work/` and `out/` to another filesystem (CI does).

`tools/` has its tests (the clean-up on a made-up copied image, the work profile's links and programs); the profile has no
test suite, so static checks matter more than usual. After any edit to the package list:

```bash
sed '/^[[:blank:]]*#.*/d;s/#.*//;/^[[:blank:]]*$/d' profile/packages.x86_64 | grep -c '[[:blank:]]$'   # must be 0
mapfile -t PKGS < <(sed '/^[[:blank:]]*#.*/d;s/#.*//;/^[[:blank:]]*$/d' profile/packages.x86_64)
pacman -Sp --print-format '%r/%n' "${PKGS[@]}"   # on a host with [blackarch] and [labyrinth]: exit 0, no provider menu
```

CI: `.github/workflows/tools.yml` checks `tools/` (fmt, clippy, tests) on every push that touches it;
`.github/workflows/build-iso.yml` builds the ISO on every push to `main` that touches `profile/`, `tools/` or the
workflow, and by hand; `archlinux:latest` privileged on `ubuntu-latest`, 14 to 21 minutes, the ISO as the artifact
`lykanos-iso`. The runner's `/` has about 14 GB free, so the job mounts the ~70 GB `/mnt` scratch disk for the work tree,
pacman cache and output; when a disk fills, the runner dies and GitHub uploads **no log** (a 22-byte zip), which is why
the build step prints free space every 30 s: keep that watcher. The job checks out this repository only, builds `tools/` and runs
`tools/target/release/lykanos-build`. Watch a
push by its full SHA: `gh run list --commit <sha>`; an empty log zip plus steps with `conclusion: None` means the runner died.

## 3. Rules

- **Privacy.** Never write a host name, network address, SSID, MAC address, serial or the owner's name into a commit or
  file; check the diff before pushing.
- **Push as you go.** Push every commit as you make it; don't ask first. Stage by explicit path; `git pull --rebase`
  before pushing.
- **Packages, not copies.** Do not vendor recipes for the `yggdrasil-*` packages, `sarina-git`, `heimdall-git` or
  `labyrinth-keyring`: they come from `[labyrinth]`, so updates reach LykanOS and Labyrinth OS the same day. Security and
  network features belong in Heimdall, not here.
- **The tool core stays small and offline-useful.** The ISO ships a hand-picked core, not `blackarch-*` category groups
  (tried and abandoned: thousands of packages, `lib32-*` conflicts, provider menus, an ISO too large for CI). Do not thin
  the core on size grounds without saying what offline capability goes. Everything past the core is Sarina's (install-time
  selection, start-menu entries): do not add tools to `packages.x86_64` to work around a gap there. Adding or removing whole
  tool categories is my product decision, not a build fix.
- **Pentest tools are LykanOS's.** LykanOS text may name them; Labyrinth OS's text never does. LykanOS's own tools live in
  a separate private repository and reach installed systems through `[lykanos]` (docs/TOOLS.md); never name that
  repository in public files.
- **Heimdall ships off** on LykanOS (`/etc/heimdall/defaults-off`, my decision 2026-11-07) until its tool rules are tested
  with the tools.
- **Licensing.** Before a publicly distributed ISO, confirm with me (Sarina's history predates its clean-room rewrite; the
  package is MPL-2.0 and an ISO ships only the built package).

## 4. Contracts

All in [docs/CONTRACTS.md](docs/CONTRACTS.md), one section each; a change says `CONTRACT:` in the commit message.

- Provided here: the OS's `ID=lykanos` (Limbo's host policy, the installer's image path, Heimdall's private microphone);
  the Yggdrasil integration files; Heimdall's `defaults-off` and `security-tools`; the `[lykanos]` repository.
- Used from others: `[labyrinth]` and its packages; Sarina's package, entry points, account name and `desktop-audit`;
  Heimdall's lock and release command (Heimdall docs/CONTRACTS.md); Yggdrasil's installer and integration format.

## 5. Open work

docs/plans/BACKLOG.md and docs/plans/LOG.md (private; ask the owner).

## 6. Know-how

- **`profile/packages.x86_64` cannot have trailing comments.** mkarchiso's parser strips the comment but keeps the
  whitespace before it, and pacman cannot resolve the name. Annotations go on their own lines (check above).
- **Pin providers explicitly.** A virtual dependency with several providers (`tessdata`, `ttf-font`, `mysql`,
  `oci-runtime`, `java-runtime`, ...) makes pacstrap print a menu and silently take option 1. New ones need an entry in
  the "explicit provider choices" section.
- **Two `pacman.conf` files, `[labyrinth]` last in both.** `profile/pacman.conf` is the build's (with `[blackarch]`);
  `profile/airootfs/etc/pacman.conf` is the live medium's and the installed system's (Arch's repositories, `[cachyos]`,
  `[labyrinth]`, `[lykanos]`). `[cachyos]` (the kernel: `linux-cachyos`, `linux-cachyos-lts` as the fallback; my decision
  2026-11-26) comes after Arch's repositories in both, CachyOS's plain one, not its x86-64-v3 builds. Last on purpose: a same-named package above wins, so they add packages and never replace the base system.
- **A `[labyrinth]` package goes into `packages.x86_64` only once Labyrinth's publish carries it**, or pacstrap stops at
  `target not found` and no ISO is built. Check Labyrinth_OS's release `packages` first.
- **No committed symlinks under `profile/airootfs/`;** add service links to `LINKS` in `tools/src/build.rs`. This checkout has
  `core.symlinks=false`, so a committed link becomes a text file and the ISO ships a broken unit.
- **Never ship `profile/airootfs/etc/passwd`, `shadow`, `group` or `gshadow`.** mkarchiso overlays airootfs without
  re-running `systemd-sysusers`, so they replace the image's accounts (greetd's `greeter` too). Accounts go in
  `airootfs/etc/sysusers.d/`.
- **The installer copies the live squashfs.** Yggdrasil's `yggdrasil-install-system` runs `unsquashfs` on `airootfs.sfs`,
  then LykanOS's clean-up in the new system. mkarchiso empties `/boot` and leaves archiso's initramfs preset and
  `mkinitcpio.conf.d/archiso.conf`; `lykanos-install-cleanup` (`tools/src/cleanup.rs`) restores both for every installed
  kernel, and keeps its own list of live-only files (`LIVE_ONLY`: add a new live-only file there as well as to `airootfs/`;
  nothing checks the two against each other). The image is found at
  `/run/archiso/bootmnt/<ID>/x86_64/airootfs.sfs` with `ID` from os-release, which must equal `install_dir` in
  `profiledef.sh` (or set `medium` in `lykanos-installer.conf`). UEFI only; on BIOS only `archinstall` installs. Changes to
  the install steps themselves belong in Yggdrasil (`packages/installer`).
- **`profile/` is a fork of archiso's `releng` profile**, kept close to upstream: copy missing pieces verbatim from
  `/usr/share/archiso/configs/releng/`; the mkinitcpio configs are exact upstream copies (a wrong hook list builds fine and
  panics at boot).
- **Start-menu entries** for tools come from `/usr/local/bin/lykanos-tool-menu` (`tools/src/bin`), run by
  `etc/pacman.d/hooks/90-lykanos-tool-menu.hook` after every pacman transaction (and while the ISO is built).
- Sarina's `assistant` integration names programs (`/usr/local/bin/labyrinth-sarina-*`) that are part of Labyrinth OS's image,
  not of the package, so on LykanOS they are missing (docs/DESKTOP.md, "Sarina in the desktop").
- A new Sarina, Heimdall or desktop reaches LykanOS when Labyrinth's publish puts it in `[labyrinth]`: the next ISO build
  and `pacman -Syu` pick it up; nothing here changes.
