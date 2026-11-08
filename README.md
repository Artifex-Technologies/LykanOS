<p align="center"><img src="assets/logo/lykanos-duo.png" width="180" alt="LykanOS logo: a wolf's head made of a maze"></p>

# LykanOS

LykanOS (named Chymaera OS, then Niflheim OS, until 2026-11-07; sometimes searched as Lycan OS) is an Arch-based distribution built for security research and
penetration testing, combining:

- **[Arch Linux](https://archlinux.org/)** as the rolling-release base
- **[BlackArch](https://github.com/BlackArch/blackarch)** tooling, available as
  a repo but not bulk-installed: the ISO ships a small offline-usable core and
  Sarina installs the rest on demand
- **[Yggdrasil](https://github.com/Artifex-Technologies/Yggdrasil)** as the
  desktop: a desktop environment written in Rust, on Hyprland, with its own top
  bar, dock, Settings, file manager, app store, task manager, Markdown reader,
  lock and login screens and installer. It is the desktop of [Labyrinth OS](https://github.com/Artifex-Technologies/Labyrinth_OS),
  and LykanOS installs the same packages. See [The desktop](#the-desktop)
- **[Heimdall](https://github.com/Artifex-Technologies/Heimdall)**, the security
  monitor Labyrinth OS uses too: installed, with rules for the security tools
  since 2026-11-07, but off until those rules are tested with the tools
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
default, which can make the base system less stable for daily use. LykanOS
aims to sit between them: an openly developed, Arch-based system with a
deliberately small default tool set, a polished desktop (Yggdrasil on
Hyprland), and an AI agent built in to help with recon, scripting, analysis, and general system
tasks.

## How tooling is delivered

The ISO ships **26 hand-picked security tools** (plus nmap, tcpdump,
testdisk, ddrescue, clonezilla and fsarchiver from releng's base list), not
BlackArch's category groups. Baking in the categories was tried and abandoned: it pulled in
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
categories that BlackArch packages don't ship. The entries are wired since
2026-11-07: a pacman hook has Sarina write one for each installed security tool,
and Yggdrasil's start menu lists them in its Security folder (not yet seen on a
booted image). The rest is still being built.

## The desktop

I decided on 2026-11-07 that LykanOS uses the same desktop as Labyrinth OS: Yggdrasil on Hyprland. Until 2026-11-07 the
desktop was KDE Plasma with SDDM, the MacTahoe theme and an Omarchy-style Hyprland session beside it; all of that is gone
from the profile.

- **Login screen:** greetd runs Yggdrasil's greeter (`yggdrasil-greeter`, in `cage`). On the live medium greetd signs the
  live user in by itself.
- **Programs:** Yggdrasil's top bar and dock, Settings (Wi-Fi, Bluetooth, sound, displays and more), Minos (files),
  Agora (the app store), the Task Manager (Ctrl + Shift + Esc), Saga MD (the Markdown reader), the lock screen, the
  password box, clipboard history, the colour picker, the calculator, spell checking and the installer. The terminal is
  Ghostty (Super + Return), the browser Firefox, and saved passwords are kept in GNOME Keyring.
- **Laptops and graphics:** Vulkan for Intel and AMD graphics, Intel video decoding, the sound mixer setups most Intel
  laptops need, and the power modes in Yggdrasil's Settings: the same set as Labyrinth OS's base list (added
  2026-11-07; my first LykanOS laptop is an Intel one).
- **Where it comes from:** the `yggdrasil-*` packages, `saga-md`, `liki-icon-theme`, `heimdall-git`, `sarina-git` and
  `labyrinth-keyring` come from Labyrinth OS's signed package repository (`[labyrinth]`, the GitHub release `packages` of
  Labyrinth_OS), built for any 64-bit x86 CPU. It is listed last, with `SigLevel = Required`, both for building the ISO
  and on the installed system, so `pacman -Syu` brings desktop updates the same day Labyrinth gets them.
- **What LykanOS adds:** four integration files Yggdrasil reads (its look, the installer's settings, Heimdall, and
  Agora's own LykanOS catalog), a pacman hook that puts the security tools in the start menu, and its look: deep teal
  with Rosewood red, a wolf's head of maze walls as the logo, and a wallpaper (`scripts/gen-source.py`,
  `scripts/gen-logo.py`, `scripts/gen-wallpaper.py`).

Details, and what is not checked yet, are in [docs/DESKTOP.md](docs/DESKTOP.md).

## Installing

The live session has an **Install LykanOS** launcher (start menu, and the live user's Desktop folder). It opens
Yggdrasil's installer window, whose install program (`yggdrasil-install-system`) is shared with Labyrinth OS: language,
keyboard, time zone, disk (optionally encrypted), account. It copies the live system to the disk, so it needs no network.
It needs a UEFI computer; the medium itself also starts on BIOS computers, where `archinstall` (command line, online) is
the way to install. See [docs/INSTALLER.md](docs/INSTALLER.md).

## Repository layout

```text
LykanOS/
├── profile/                  # archiso profile used to build the live/install ISO
│   ├── profiledef.sh         # ISO metadata (name, label, boot modes, etc.)
│   ├── packages.x86_64       # package list installed into the ISO/target system
│   ├── pacman.conf           # repos for the build: core/extra/multilib + [blackarch] + [labyrinth]
│   ├── syslinux/, efiboot/   # releng's boot menus (BIOS and UEFI) with LykanOS's names
│   └── airootfs/             # files overlaid onto the live system's root (greetd, Yggdrasil integration files, the
│                             #   installed system's pacman.conf, lykanos-install-cleanup, lykanos-tool-menu and its
│                             #   pacman hook, Heimdall's switch files, Agora's LykanOS catalog, logo, icon, wallpaper)
├── assets/logo/              # the logo: my picture (lykanos-original.jpg), its straightened outline and the colour set
├── docs/                     # ARCHITECTURE.md, DESKTOP.md (Yggdrasil), INSTALLER.md, YGGDRASIL_MOVE.md
├── scripts/                  # build.sh (the ISO build); gen-source.py, gen-logo.py, gen-wallpaper.py (the look)
├── CLAUDE.md                 # Guidance for AI agents working in this repo
├── THIRD_PARTY_NOTICES.md    # what is adapted from other projects
└── .github/workflows/        # CI: builds the ISO and keeps it as a workflow artifact
```

## Building the ISO

Building requires an Arch Linux (or Arch container) environment with
`archiso` installed — it will not run on Windows/macOS directly. See
[scripts/build.sh](scripts/build.sh) and
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

```bash
sudo pacman -S --needed archiso curl git
git clone https://github.com/Artifex-Technologies/LykanOS.git
cd LykanOS

# Build the ISO as root (mkarchiso requires it)
sudo bash scripts/build.sh
```

Nothing is built locally: every package comes from a repository. `scripts/build.sh` installs BlackArch's key (BlackArch's
`strap.sh`), trusts the Labyrinth package key (fetched, then checked against its full fingerprint before it is signed
locally), makes the systemd service links in a work copy of the profile and runs `mkarchiso`. Until 2026-11-07 a first
step built `sarina-git` into a local `[niflheim]` repository from `packages/sarina`, a copy of Sarina that had been
removed on 2026-10-08.

The resulting ISO is written to `out/`. Set `LYKANOS_WORK_DIR` and
`LYKANOS_OUT_DIR` to put the build tree and output on a different filesystem
(CI uses this to reach a larger disk). CI keeps each ISO it builds as the
workflow artifact `lykanos-iso`.

### Building under WSL

On Windows, WSL2 with an Arch distribution works. **Clone into the WSL
filesystem** (`~/`), never `/mnt/c`: Windows drives don't preserve Unix
permissions, and a build tree there would also try to sync to OneDrive.

## Status

Early. Since 2026-11-07 the profile installs Yggdrasil instead of KDE Plasma.

**The ISO builds; it has not been booted yet.** The GitHub workflow builds an ISO on every push to `main` that touches
`profile/`, `scripts/` or the workflow itself, and when started by hand. From 2026-10-08, when the repository moved here,
every run stopped before `mkarchiso` (the scripts had no executable bit, and the local Sarina build had no source); since
7d2bfe5 (2026-11-07) the runs build an ISO. The latest, `lykanos-2026.11.08-x86_64.iso` (27f3064, 2026-11-08), is
4.2 GiB with 874 packages, and it is the first with an installer that can install: the ISOs of 2026-11-07 carried the
`yggdrasil-installer` published on 2026-11-06, which has only the window. The runs in between (b81ecc2 to d0dc393)
stopped at `target not found: yggdrasil-taskmanager`, until Labyrinth's publish on 2026-11-08 carried the Task Manager.

Booting the ISO in a VM, installing and checking the result is step 6 of
[docs/YGGDRASIL_MOVE.md](docs/YGGDRASIL_MOVE.md), and it is still open.

The installer has never been run on LykanOS. The same install program has installed Labyrinth OS in VMs. Install speed
has not been measured, and there is no xz build to compare the zstd ISO's size with.

Not yet done: the agent-assisted install flow, remapping BlackArch's own menu categories, testing Heimdall's tool rules
with the tools (Heimdall stays off until then), a boot menu theme, and testing on real hardware (a GPU, a laptop).
Expect rapid changes.

## Sarina

[Sarina](https://github.com/Artifex-Technologies/Sarina) (formerly
Chymaera Code Agent) is a local-first agent written in Go, with an optional
Python helper, runnable against local open models. LykanOS installs it as
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

Sarina's intended role in the running system (only part of Organisation
exists yet):

- **Guided install** — ask what the machine is for and install tooling to
  match, instead of shipping a guess.
- **Organisation** — generate `.desktop` entries and start-menu categories
  for tools that drop bare binaries into `/usr/bin`, which is the problem
  Kali's curated menus solve by hand. Generating them is wired since
  2026-11-07 (`lykanos-tool-menu`, run by a pacman hook, calls Sarina's
  `desktop-audit`); remapping the `X-BlackArch-*` categories of tools that ship
  their own entry is not done.
- **Analysis** — read scan, capture and log output in place and summarise it,
  so triage doesn't mean scrolling raw tool output.
- **Maintenance** — evaluate `pacman -Syu` and BlackArch updates for what is
  safe to apply automatically versus what needs a human look.

A newer Sarina reaches LykanOS through `pacman -Syu`, like the desktop.

## References

- [Yggdrasil](https://github.com/Artifex-Technologies/Yggdrasil) — the desktop
  environment (its README lists the programs and keyboard shortcuts;
  `docs/INTEGRATIONS.md` and `docs/INSTALLER.md` describe what LykanOS plugs
  into).
- [Labyrinth OS](https://github.com/Artifex-Technologies/Labyrinth_OS) — the
  sister system whose signed package repository (`docs/PACKAGE_REPO.md` there)
  supplies the desktop, Heimdall, Sarina and the Labyrinth key.
- [Hyprland](https://hyprland.org) — the compositor Yggdrasil's session runs on.
- [greetd](https://sr.ht/~kennylevinsen/greetd/) — the login manager that
  starts Yggdrasil's greeter.
- [Heimdall](https://github.com/Artifex-Technologies/Heimdall) — the security
  monitor.
- [Limbo](https://github.com/Artifex-Technologies/Limbo) — the sandbox for
  guest programs in virtual machines; in `[labyrinth]`, not on the ISO. It never
  gives a guest real hardware on a LykanOS host (see [docs/DESKTOP.md](docs/DESKTOP.md), "Limbo").
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
