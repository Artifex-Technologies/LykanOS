# Moving Niflheim OS to Yggdrasil

Decided 2026-11-07: Niflheim OS (formerly Chymaera OS) leaves KDE, SDDM and Calamares and uses the same desktop as Labyrinth OS:
Yggdrasil on Hyprland, greetd with Yggdrasil's greeter, and Yggdrasil's installer. The repo was renamed to
`Artifex-Technologies/Niflheim_OS` the same day, and every Chymaera name inside it became Niflheim.

Niflheim stays a security and research system: its tools layer (the baked-in core and the BlackArch groups, `docs/ARCHITECTURE.md`)
is unchanged. Gaming pieces stay Labyrinth's (no Steam, Proton, frame generation).

## Where the desktop comes from

Not copies: packages. Labyrinth's signed package repository (the GitHub release `packages` of Labyrinth_OS, Labyrinth's
`docs/PACKAGE_REPO.md`) already builds every Yggdrasil package, Heimdall and Limbo. Niflheim adds it to its `pacman.conf` (signed,
`SigLevel = Required`, last) and ships `labyrinth-keyring` so the installed system trusts it from its first boot. Updates to the
desktop then reach both systems the same day.

## Steps

1. **Packages.** In `profile/packages.x86_64`: remove `plasma-meta`, `sddm`, `dolphin`, `konsole`, `kwin` and other KDE pieces,
   `waybar`, `mako`, `nwg-dock-hyprland`, `kvantum`, `calamares`, the MacTahoe theme; add `hyprland`, `greetd`, `ghostty`,
   `labyrinth-keyring`, `yggdrasil-session`, `-panel`, `-settings`, `-minos`, `-portal`, `-polkit`, `-lock`, `-idle`,
   `-keyring-sync`, `-xmove`, `-clipboard`, `-colorpicker`, `-calculator`, `-spell`, `-agora`, `-installer`, `-theme`,
   `liki-icon-theme`, `heimdall-git`, `gnome-keyring` (saved passwords stay on GNOME Keyring, decided 2026-11-07).
2. **Live image (`profile/airootfs`).** Remove `etc/calamares`, `etc/sddm.conf.d`, `etc/xdg/kscreenlockerrc`, the KDE files in
   `home/live/.config` and the Omarchy/Waybar modes (`niflheim-desktop-mode` goes too). Add `etc/greetd/config.toml` and `live.toml`
   (the live user signed in by the greeter, as Labyrinth's), `display-manager.service` -> greetd, and Yggdrasil's integration files
   under `usr/share/yggdrasil/integrations/`: `niflheim-branding.conf` (name, logo), `niflheim-installer.conf` (kind installer),
   `niflheim-security.conf` (Heimdall).
3. **Installer.** Yggdrasil's installer window needs an install program (`kind = installer`, `system = ...`). Labyrinth's is
   `labyrinth-install-system` (copy the medium, encryption, btrfs subvolumes, users, GRUB, then the cleanup and bootfinish
   helpers). Make it shareable rather than forking it: move it into a package both systems install (for example an
   `os-installer-system` package in the Labyrinth repo that reads the OS name and helpers from its integration file), then point
   `niflheim-installer.conf` at it. `niflheim-install-cleanup` keeps Niflheim's own clean-up steps.
4. **Build.** `scripts/build.sh` adds the Labyrinth key by fingerprint (as Labyrinth's build does) before `mkarchiso`. The local
   `[niflheim]` repo keeps only what is Niflheim's own (sarina-git while it is vendored).
5. **Docs.** `README.md`, `CLAUDE.md`, `docs/DESKTOP.md`, `docs/INSTALLER.md`, `docs/ARCHITECTURE.md` describe KDE, Waybar,
   Omarchy modes and Calamares: rewrite them once steps 1-4 are in.
6. **Check.** Build the ISO, boot it in a VM, install, reboot, sign in, open Settings, Agora and a terminal; `pacman -Syu` pulls
   from both repositories; Heimdall runs.

Steps 1, 2, 4 and 5 are edits in this repo. Step 3 changes Labyrinth too. Step 6 needs an ISO build and a VM: it waits until
the documentation catch-up (Labyrinth `docs/CATCH_UP_2026-11-07.md`) is done, the same hold as Labyrinth's next ISO.
