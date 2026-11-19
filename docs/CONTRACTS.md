# Contracts with other repositories

What other projects of the Artifex Ecosystem rely on from LykanOS, and what LykanOS relies on from them. One section per
contract: what, who uses it, where it lives. A change to any of them says `CONTRACT:` in the commit message, changes this
file in the same commit, and is told to the other repository's maintainers.

## The OS's ID: `ID=lykanos`

- **What:** `/etc/os-release` says `ID=lykanos` and `LOGO=lykanos`. The ID equals `install_dir` in `profiledef.sh`, so the
  installer finds the image at `/run/archiso/bootmnt/lykanos/x86_64/airootfs.sfs`.
- **Who:** Limbo's host policy (refuses PCI and USB passthrough on `ID=lykanos`, and on the old `chymaera`); Yggdrasil's
  installer (the image path); Heimdall's private microphone (`heimdall-private-mic os-setup` links Sarina's raw-microphone
  option only where `ID=lykanos`); Yggdrasil (the logo).
- **Where:** `profile/airootfs/etc/os-release`, `profile/profiledef.sh`.

## Yggdrasil integration files

- **What:** files in `/usr/share/yggdrasil/integrations/` in Yggdrasil's integration format (Yggdrasil's
  `docs/INTEGRATIONS.md`): `lykanos-branding.conf` (the accent colour, the wallpaper), `lykanos-installer.conf` (kind
  installer: `command = /usr/bin/yggdrasil-install-system`, `cleanup = /usr/local/bin/lykanos-install-cleanup`,
  `reserved_logins = heimdall sarina`, the new account's groups), `lykanos-security.conf` (Heimdall's API, lock, release and
  device commands), `lykanos-packages.conf`.
- **Who:** Yggdrasil (Settings, the panel, the installer, Agora).
- **Where:** `profile/airootfs/usr/share/yggdrasil/integrations/`.

## Heimdall's switches on the image

- **What:** `/etc/heimdall/defaults-off` (Heimdall's install hook leaves it off; `heimdall-enable-defaults --force` switches
  it on) and `/etc/heimdall/security-tools` (Heimdall's tool rules summarise the security tools' activity instead of
  alerting on it). Both are Heimdall's contracts (its docs/CONTRACTS.md); LykanOS provides the files.
- **Who:** Heimdall.
- **Where:** `profile/airootfs/etc/heimdall/`.

## The `[lykanos]` package repository

- **What:** the release named `packages` of this repository is a signed pacman repository (signed with Labyrinth's key,
  which `labyrinth-keyring` already trusts), listed last in the installed system's `pacman.conf`. It carries LykanOS's own
  security tools, built from a separate private repository. [TOOLS.md](TOOLS.md).
- **Who:** installed LykanOS systems.
- **Where:** `profile/airootfs/etc/pacman.conf`, the GitHub release `packages`.

## What LykanOS uses from others

- **`[labyrinth]`** (Labyrinth OS's signed repository, last in both `pacman.conf` files): the Yggdrasil packages,
  `sarina-git`, `heimdall-git`, `heimdall-private-mic-git`, `heimdall-multilink-git`, `labyrinth-keyring`. The key is
  trusted by its full fingerprint in `scripts/build.sh`.
- **Sarina** (`sarina-git`): the package name; `sarina`, `sarina-gui` and `sarina-service` on `PATH`; the `sarina` account
  name (reserved in `lykanos-installer.conf`); `sarina desktop-audit --apply --only-category Security --target-dir
  /usr/local/share/applications`, run by `/usr/local/bin/lykanos-tool-menu` from a pacman hook.
- **Heimdall**: its lock `/etc/heimdall/contained` and `heimdalld contain release`, named in `lykanos-security.conf`.
- **Yggdrasil**: `yggdrasil-install-system` and its integration format; the start menu's Security folder (freedesktop
  categories).
