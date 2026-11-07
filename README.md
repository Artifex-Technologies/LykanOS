# Niflheim OS

Niflheim OS (formerly Chymaera OS; renamed 2026-11-07) is an Arch-based distribution built for security research and
penetration testing, combining:

- **[Arch Linux](https://archlinux.org/)** as the rolling-release base
- **[BlackArch](https://github.com/BlackArch/blackarch)** tooling, available as
  a repo but not bulk-installed: the ISO ships a small offline-usable core and
  Sarina installs the rest on demand
- **[Yggdrasil](https://github.com/Artifex-Technologies/Yggdrasil)** as the
  desktop: a desktop environment written in Rust, on Hyprland, with its own top
  bar, dock, Settings, file manager, app store, lock and login screens and
  installer. It is the desktop of [Labyrinth OS](https://github.com/Artifex-Technologies/Labyrinth_OS),
  and Niflheim installs the same packages. See [The desktop](#the-desktop)
- **[Heimdall](https://github.com/Artifex-Technologies/Heimdall)**, the security
  monitor Labyrinth OS uses too, running in the live session and on the
  installed system
- **[Sarina](https://github.com/Artifex-Technologies/Sarina)**, an AI agent integrated
  directly into the OS — for choosing and organising tooling, and for making
  sense of scan and capture output in place

The goal is Kali-style organization, documentation, and out-of-the-box UX for
security tooling, on top of Arch's package freshness and BlackArch's tool
catalog — built and versioned in the open on GitHub, which BlackArch and Kali
do not do.

## Why another security distro?

Kali does a lot right: curated tool menus, consistent documentation, a
predictable release/support cadence. But Kali isn't developed in the open on
GitHub, and BlackArch (which *is* on GitHub) ships its full tool catalog by
default, which can make the base system less stable for daily use. Niflheim
aims to sit between them: an openly developed, Arch-based system with a
deliberately small default tool set, a polished desktop (Yggdrasil on
Hyprland), and an AI agent built in to help with recon, scripting, analysis, and general system
tasks.

## How tooling is delivered

The ISO ships **~27 hand-picked security tools**, not BlackArch's category
groups. Baking in the categories was tried and abandoned: it pulled in
several thousand packages nobody had chosen, along with unresolvable
32-bit dependencies and provider conflicts that made the ISO too large to
build in CI. The reasoning is in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

What ships covers recon, traffic capture, wireless, password attacks, web
application testing, exploitation, reverse engineering and forensics — chosen
so that **the live session is useful with no network at all.** Air-gapped
client sites and offline forensics are where a live USB earns its keep, and
they are the one thing an install-on-demand model cannot serve.

Everything past that core is Sarina's job: asking what the machine is for,
installing accordingly, and generating the `.desktop` entries and start-menu
categories that BlackArch packages don't ship. Those capabilities are still
being built.

## The desktop

I decided on 2026-11-07 that Niflheim uses the same desktop as Labyrinth OS: Yggdrasil on Hyprland. Until 2026-11-07 the
desktop was KDE Plasma with SDDM, the MacTahoe theme and an Omarchy-style Hyprland session beside it; all of that is gone
from the profile.

- **Login screen:** greetd runs Yggdrasil's greeter (`yggdrasil-greeter`, in `cage`). On the live medium greetd signs the
  live user in by itself.
- **Programs:** Yggdrasil's top bar and dock, Settings (Wi-Fi, Bluetooth, sound, displays and more), Minos (files),
  Agora (the app store), the lock screen, the password box, clipboard history, the colour picker, the calculator, spell
  checking and the installer. The terminal is Ghostty (Super + Return), the browser Firefox, and saved passwords are kept
  in GNOME Keyring.
- **Where it comes from:** the `yggdrasil-*` packages, `heimdall-git`, `sarina-git` and `labyrinth-keyring` come from
  Labyrinth OS's signed package repository (`[labyrinth]`, the GitHub release `packages` of Labyrinth_OS), built for any 64-bit x86 CPU.
  It is listed last, with `SigLevel = Required`, both for building the ISO and on the installed system, so `pacman -Syu`
  brings desktop updates the same day Labyrinth gets them.
- **What Niflheim adds:** three integration files Yggdrasil reads: the accent colour, the installer's settings and
  Heimdall.

Details, and what is not checked yet, are in [docs/DESKTOP.md](docs/DESKTOP.md).

## Installing

The live session has an **Install Niflheim** launcher (start menu, and the live user's Desktop folder). It opens
Yggdrasil's installer window, whose install program (`yggdrasil-install-system`) is shared with Labyrinth OS: language,
keyboard, time zone, disk (optionally encrypted), account. It copies the live system to the disk, so it needs no network.
It needs a UEFI computer; the medium itself also starts on BIOS computers, where `archinstall` (command line, online) is
the way to install. See [docs/INSTALLER.md](docs/INSTALLER.md).

## Repository layout

```
Niflheim_OS/
├── profile/                  # archiso profile used to build the live/install ISO
│   ├── profiledef.sh         # ISO metadata (name, label, boot modes, etc.)
│   ├── packages.x86_64       # package list installed into the ISO/target system
│   ├── pacman.conf           # repos for the build: core/extra/multilib + [blackarch] + [labyrinth]
│   └── airootfs/             # files overlaid onto the live system's root (greetd, Yggdrasil integration files, the
│                             #   installed system's pacman.conf, niflheim-install-cleanup)
├── docs/                     # ARCHITECTURE.md, DESKTOP.md (Yggdrasil), INSTALLER.md, YGGDRASIL_MOVE.md
├── scripts/build.sh          # the ISO build
├── CLAUDE.md                 # Guidance for AI agents working in this repo
└── .github/workflows/        # CI: build (and eventually publish) the ISO
```

## Building the ISO

Building requires an Arch Linux (or Arch container) environment with
`archiso` installed — it will not run on Windows/macOS directly. See
[scripts/build.sh](scripts/build.sh) and
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

```bash
sudo pacman -S --needed archiso curl git
git clone https://github.com/Artifex-Technologies/Niflheim_OS.git
cd Niflheim_OS

# Build the ISO as root (mkarchiso requires it)
sudo bash scripts/build.sh
```

Nothing is built locally: every package comes from a repository. `scripts/build.sh` installs BlackArch's key (BlackArch's
`strap.sh`), trusts the Labyrinth package key (fetched, then checked against its full fingerprint before it is signed
locally), makes the systemd service links in a work copy of the profile and runs `mkarchiso`. Until 2026-11-07 a first
step built `sarina-git` into a local `[niflheim]` repository from `packages/sarina`, a copy of Sarina that had been
removed on 2026-10-08.

The resulting ISO is written to `out/`. Set `NIFLHEIM_WORK_DIR` and
`NIFLHEIM_OUT_DIR` to put the build tree and output on a different filesystem
(CI uses this to reach a larger disk).

### Building under WSL

On Windows, WSL2 with an Arch distribution works. **Clone into the WSL
filesystem** (`~/`), never `/mnt/c`: Windows drives don't preserve Unix
permissions, and a build tree there would also try to sync to OneDrive.

## Status

Early. Since 2026-11-07 the profile installs Yggdrasil instead of KDE Plasma, and the package list resolves against Arch,
BlackArch and `[labyrinth]` (checked when the change was made).

**No ISO of this setup has been booted yet.** The GitHub workflow builds an ISO on every push to `main` that touches
`profile/` or `scripts/`. Until 2026-11-07 every run since the repository moved here (2026-10-08) stopped before
`mkarchiso` (the scripts had no executable bit, and the local Sarina build had no source); the first run without those
problems started on 2026-11-07. Booting the ISO in a VM, installing and checking the result is step 6 of
[docs/YGGDRASIL_MOVE.md](docs/YGGDRASIL_MOVE.md), and it is still open. It waits for `[labyrinth]` to carry a
`yggdrasil-installer` with the install program (Labyrinth's daily publish on 2026-11-08).

The installer has never been run on Niflheim. The same install program has installed Labyrinth OS in VMs. Install speed
and the larger ISO that zstd compression produces have not been measured.

Not yet done: the agent-assisted install flow, tool categorisation, Niflheim's own look beyond the accent colour (logo,
wallpaper, boot menu), and testing on real GPU hardware. Expect rapid changes.

## Sarina

[Sarina](https://github.com/Artifex-Technologies/Sarina) (formerly
Niflheim Code Agent) is a local-first agent written in Go, with an optional
Python helper, runnable against local open models. Niflheim installs it as
`sarina-git` from `[labyrinth]`, the same package Labyrinth OS installs (its
recipe is Labyrinth's `packages/pkgbuilds/sarina-git`). Sarina's repository
ships its own Arch packaging (`packaging/arch/`) and a
[`blackarch_compat`](https://github.com/Artifex-Technologies/Sarina/tree/main/blackarch_compat)
harness built specifically for *this* distro: it installs BlackArch's
`blackarch-*` pacman categories one at a time in disposable Docker containers
to catch install-time breakage. Now that the ISO no longer bakes in those
categories, that harness becomes the safety check behind Sarina's own
install-time suggestions — it should know a category installs cleanly before
offering it.

Sarina's intended role in the running system, none of which exists yet:

- **Guided install** — ask what the machine is for and install tooling to
  match, instead of shipping a guess.
- **Organisation** — generate `.desktop` entries and start-menu categories
  for tools that drop bare binaries into `/usr/bin`, which is the problem
  Kali's curated menus solve by hand.
- **Analysis** — read scan, capture and log output in place and summarise it,
  so triage doesn't mean scrolling raw tool output.
- **Maintenance** — evaluate `pacman -Syu` and BlackArch updates for what is
  safe to apply automatically versus what needs a human look.

A newer Sarina reaches Niflheim through `pacman -Syu`, like the desktop.

## References

- [Yggdrasil](https://github.com/Artifex-Technologies/Yggdrasil) — the desktop
  environment (its README lists the programs and keyboard shortcuts;
  `docs/INTEGRATIONS.md` and `docs/INSTALLER.md` describe what Niflheim plugs
  into).
- [Labyrinth OS](https://github.com/Artifex-Technologies/Labyrinth_OS) — the
  sister system whose signed package repository (`docs/PACKAGE_REPO.md` there)
  supplies the desktop, Heimdall, Sarina and the Labyrinth key.
- [Hyprland](https://hyprland.org) — the compositor Yggdrasil's session runs on.
- [greetd](https://sr.ht/~kennylevinsen/greetd/) — the login manager that
  starts Yggdrasil's greeter.
- [Heimdall](https://github.com/Artifex-Technologies/Heimdall) — the security
  monitor.
- [archinstall](https://github.com/archlinux/archinstall) — Arch's guided
  CLI installer; kept available as a scripted alternative to the graphical installer.
- [archiso](https://github.com/archlinux/archiso) — the tool used to build
  this project's live/install ISO (see `profile/`).
- [BlackArch](https://github.com/BlackArch/blackarch) — source of the
  security tooling repo and package catalog.
- [Sarina](https://github.com/Artifex-Technologies/Sarina) — the AI agent
  integrated into the OS (`sarina-git` from `[labyrinth]`).

## License

[GPL-3.0](LICENSE), matching the convention used by BlackArch and most
security-tooling distributions.
