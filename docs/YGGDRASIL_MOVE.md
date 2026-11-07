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

Steps 1-5 were done on 2026-11-07; step 6 is open.

1. [x] **Packages.** Done 2026-11-07 (3193bca). In `profile/packages.x86_64`: remove `plasma-meta`, `sddm`, `dolphin`,
   `konsole`, `kwin` and other KDE pieces, `waybar`, `mako`, `nwg-dock-hyprland`, `kvantum`, `calamares`, the MacTahoe theme;
   add `hyprland`, `greetd`, `ghostty`, `labyrinth-keyring`, `yggdrasil-session`, `-panel`, `-settings`, `-minos`, `-portal`,
   `-polkit`, `-lock`, `-idle`, `-keyring-sync`, `-xmove`, `-clipboard`, `-colorpicker`, `-calculator`, `-spell`, `-agora`,
   `-installer`, `-theme`, `liki-icon-theme`, `heimdall-git`, `gnome-keyring` (saved passwords stay on GNOME Keyring, decided
   2026-11-07). Bluetooth (`bluez`), `ufw` and `clamav` came in too.
2. [x] **Live image (`profile/airootfs`).** Done 2026-11-07 (3193bca). Remove `etc/calamares`, `etc/sddm.conf.d`,
   `etc/xdg/kscreenlockerrc`, the KDE files in `home/live/.config` and the Omarchy/Waybar modes (`niflheim-desktop-mode` goes
   too). Add `etc/greetd/config.toml` and `live.toml` (the live user signed in by the greeter, as Labyrinth's),
   `display-manager.service` -> greetd, and Yggdrasil's integration files under `usr/share/yggdrasil/integrations/`:
   `niflheim-branding.conf` (the accent colour; the name comes from os-release), `niflheim-installer.conf` (kind installer),
   `niflheim-security.conf` (Heimdall). Also added: the installed system's `etc/pacman.conf` with `[labyrinth]`,
   `pacman-init.service` and the installer launchers.
3. [x] **Installer.** Done 2026-11-07, differently from the first plan: instead of a new `os-installer-system` package,
   Labyrinth's `labyrinth-install-system` moved into Yggdrasil as `yggdrasil-install-system`, shipped in the
   `yggdrasil-installer` package and configured by each OS's integration file (Yggdrasil 0c54aa0, Labyrinth 2b18499).
   `niflheim-installer.conf` points at it and `niflheim-install-cleanup` keeps Niflheim's own clean-up steps (3193bca).
4. [x] **Build.** Done 2026-11-07 (3193bca). `scripts/build.sh` adds the Labyrinth key by its full fingerprint before
   `mkarchiso` and makes the systemd service links in the work copy (they had been committed as text files). The local
   `[niflheim]` repo keeps only what is Niflheim's own (sarina-git while it is vendored).
5. [x] **Docs.** Done 2026-11-07. `README.md`, `CLAUDE.md`, `docs/DESKTOP.md`, `docs/INSTALLER.md`, `docs/ARCHITECTURE.md`
   and `THIRD_PARTY_NOTICES.md` describe Yggdrasil, greetd and Yggdrasil's installer instead of KDE, Waybar, the Omarchy
   modes and Calamares.
6. [ ] **Check.** Build the ISO, boot it in a VM, install, reboot, sign in, open Settings, Agora and a terminal;
   `pacman -Syu` pulls from both repositories; Heimdall runs. Before it can run:
   - CI has to get past "Build local packages": the scripts are stored without the executable bit, and the tree has no
     gitlink for `packages/sarina` (only `.gitmodules`).
   - `[labyrinth]` needs a `yggdrasil-installer` built after Yggdrasil 0c54aa0: the one published on 2026-11-06 has only
     the window, not `yggdrasil-install-system`.

   Also check in the VM: whether Heimdall runs on the live medium (its package's pacman hook switches it on when the image
   is built), and whether the screen lock can strand the passwordless live session (`docs/DESKTOP.md`, "Not checked
   yet"). Use a UEFI VM: the installer needs UEFI.

Step 6 needs an ISO build and a VM: it waits until the documentation catch-up (Labyrinth `docs/CATCH_UP_2026-11-07.md`) is
done, the same hold as Labyrinth's next ISO.
