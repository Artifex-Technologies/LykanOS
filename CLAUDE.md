# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Push every commit as you make it; don't ask me first. Before pushing, check the diff names no host name, network address, SSID
or MAC address.

Since 2026-11-07 the desktop is Yggdrasil on Hyprland, with greetd and Yggdrasil's installer, the same as Labyrinth OS's
(until then KDE Plasma, SDDM and Calamares). `docs/YGGDRASIL_MOVE.md` records the move; its step 6, building the ISO and
checking it in a VM, is still open.

## What this repo is

LykanOS (named Niflheim OS until 2026-11-07, Chymaera OS before that) is an Arch-based security distribution. This repo contains **no
application code** — it is an [archiso](https://github.com/archlinux/archiso)
profile plus the build glue that turns it into a bootable ISO. Nothing is
built here: the desktop (Yggdrasil), Sarina, Heimdall and `labyrinth-keyring`
are packages from Labyrinth OS's signed repository (`[labyrinth]`); their code
and recipes live in the Yggdrasil, Sarina, Heimdall and Labyrinth_OS repos.

Read `docs/ARCHITECTURE.md` before making design decisions; it records *why*
the current structure exists (a small baked-in tool core over BlackArch
category groups, the desktop as packages rather than copies, a graphical
installer over `archinstall`, etc.). `docs/DESKTOP.md` covers the Yggdrasil
desktop and what LykanOS adds to it, `docs/INSTALLER.md` the install flow;
both separate what was verified from what was not.

## Where the build runs

`mkarchiso` requires Arch Linux and root. Changes are validated on an Arch host
or container (on Windows: WSL with an Arch distro, cloned into the ext4 side,
**not** `/mnt/c`, or permissions break), or by pushing and reading the GitHub
Actions run.

Because of that, static verification matters more than usual here. Prefer
simulating what a tool will do over assuming it works — see the
`packages.x86_64` note below for a concrete example.

## Build commands

One step, as root (`mkarchiso` requires it):

```bash
sudo bash scripts/build.sh
```

`scripts/build.sh` installs BlackArch's key (BlackArch's `strap.sh`), trusts
the Labyrinth package key by its full fingerprint
(`C9CE784C8BD9B661042075972D3BD4B1E7CD07A5`: fetched, fingerprint checked, then
`pacman-key --lsign-key`), makes the systemd service links in a work copy of
the profile (see below) and runs `mkarchiso`. Until 2026-11-07 a first,
non-root step built `sarina-git` into a local `[niflheim]` repo from
`packages/sarina`, a copy of Sarina that had been removed on 2026-10-08.

`scripts/build.sh` honours `LYKANOS_WORK_DIR` and `LYKANOS_OUT_DIR` for build
hosts where `work/` and `out/` must sit on a different filesystem; CI uses these
to move the build off the runner's small root disk.

There is no test suite, linter, or build step for this repo itself. Sarina,
Yggdrasil and Heimdall have their own, in their repositories.

## Things that will bite you

**`profile/packages.x86_64` cannot have trailing comments.** mkarchiso parses
it with `sed '/^[[:blank:]]*#.*/d;s/#.*//;/^[[:blank:]]*$/d'`, which strips the
comment but leaves the whitespace before it, yielding a package name with
trailing spaces that pacman cannot resolve. Put annotations on their own
lines. Verify any edit by simulating the real parser:

```bash
sed '/^[[:blank:]]*#.*/d;s/#.*//;/^[[:blank:]]*$/d' profile/packages.x86_64 | grep -c '[[:blank:]]$'   # must be 0
```

**Pin providers explicitly.** Virtual dependencies with multiple providers
(`tessdata`, `ttf-font`, `mysql`, `oci-runtime`, …) make pacstrap print a
numbered menu and then silently take option 1, which is often absurd
(`tesseract-data-afr` for `tessdata`). New packages that introduce a provider
choice need an explicit entry in the "explicit provider choices" section.

**There are two `pacman.conf` files, and `[labyrinth]` is last in both.**
`profile/pacman.conf` is for the build (with `[blackarch]`);
`profile/airootfs/etc/pacman.conf` is the live medium's and the installed
system's (Arch's repos and `[labyrinth]` only). `[labyrinth]` has
`SigLevel = Required` and is last on purpose: a package of the same name in a
repository above it wins, so it can add packages but never replace the base
system. Do not vendor recipes for the `yggdrasil-*` packages, `sarina-git`,
`heimdall-git` or `labyrinth-keyring` into this repo: they come from
`[labyrinth]`, so updates reach LykanOS and Labyrinth the same day.

**Do not commit symlinks under `profile/airootfs/`; add service links to
`scripts/build.sh`.** This checkout has `core.symlinks=false`, so a committed
link becomes a text file holding its target, and the ISO then ships a text file
where systemd expects a symlink: a broken unit. Until 2026-11-07 the default
target, the display manager, NetworkManager and the `systemd-firstboot` mask
were such files. `scripts/build.sh` now makes every link in the work copy
(`default.target`, `display-manager.service` -> greetd, NetworkManager,
Bluetooth, `pacman-init.service`, the `systemd-firstboot` mask). A new enabled
unit or mask goes there too.

**Never ship `profile/airootfs/etc/passwd`, `shadow`, `group` or `gshadow`.**
Releng does, and copying that pattern looks right — but mkarchiso overlays
airootfs onto the pacstrapped root without re-running `systemd-sysusers`, so
those files *replace* rather than merge. You would delete every service
account in the image, greetd's `greeter` included, and the login screen would
never start. Add accounts through `airootfs/etc/sysusers.d/` instead.

**The installer copies the live squashfs; it does not pacstrap, and the live
image is not an installable root.** Yggdrasil's `yggdrasil-install-system`
(shared with Labyrinth OS; its code is in the Yggdrasil repo, not here) runs
`unsquashfs` on `airootfs.sfs` and then LykanOS's clean-up inside the new
system (`lykanos-installer.conf` names it; details in `docs/INSTALLER.md`).
Traps, each of which produces an install that "succeeds" and then fails later:

- mkarchiso deletes everything in `/boot` from the image, so the target has no
  kernel, and the image's initramfs preset and `mkinitcpio.conf.d/archiso.conf`
  are archiso's. `lykanos-install-cleanup` restores both. Do not remove it.
- That script keeps its own list of live-only files (sudoers, polkit, greetd's
  `live.toml` and its drop-in, the installer launcher and integration file,
  ...). Add any new live-only file to it as well as to `airootfs/`; nothing
  checks the two against each other.
- The program finds the image at `/run/archiso/bootmnt/<ID>/x86_64/airootfs.sfs`,
  with `ID` from `/etc/os-release` (`lykanos`), which must equal `install_dir`
  in `profiledef.sh` (not archiso's `arch`). Change one and you must change the
  other, or set `medium` in `lykanos-installer.conf`.
- The program installs on UEFI computers only. The medium also boots BIOS
  (syslinux); there only `archinstall` installs.
- Changes to the install steps themselves belong in Yggdrasil
  (`packages/installer`, its `docs/INSTALLER.md`); LykanOS's part is the
  integration file and the clean-up. Nothing of it has been run on LykanOS
  yet.

**`profile/` is a fork of archiso's `releng` profile**, kept deliberately close
to upstream. When something is missing, copy it verbatim from
`/usr/share/archiso/configs/releng/` (or the archiso GitLab) rather than
writing it from memory — the mkinitcpio configs in
`profile/airootfs/etc/mkinitcpio.*` are exact upstream copies for this reason,
and getting their hook list subtly wrong produces an ISO that builds fine and
then panics at boot.

## CI

`.github/workflows/build-iso.yml` runs `archlinux:latest` privileged on
`ubuntu-latest`. Two constraints shape it:

- **Disk.** The runner's `/` has ~14 GB free, nowhere near enough for a
  BlackArch + desktop pacstrap. The job mounts the runner's ~70 GB `/mnt`
  scratch disk into the container and puts the work tree, pacman cache, and
  ISO output there. When `/` or `/mnt` fills, the runner agent is killed and
  GitHub uploads **no log at all** for the job — a 22-byte empty zip. The
  build step therefore prints free space every 30s so the log tail always
  shows which filesystem ran out. Keep that watcher.
- **Nothing to build first.** The job checks out this repo only (no
  submodule, no token) and runs `bash ./scripts/build.sh`.

Until 2026-11-07 (7d2bfe5) no run got as far as `mkarchiso`: the scripts were
stored without the executable bit, and the local Sarina build had no source.
Keep `scripts/build.sh` at mode `100755` in git (`git ls-files -s`), and keep CI
calling it through `bash` so a lost bit cannot stop the build again.

Diagnosing a failed run without `gh` installed: the REST API works with a PAT
carrying Actions: read. An empty log zip plus steps with `conclusion: None`
means the runner died rather than the build failing.

## Tool delivery model

The ISO ships a **small hand-picked core** of security tools, not
`blackarch-*` category groups. The groups were tried and abandoned — they
pulled in thousands of packages, unresolvable `lib32-*` deps, provider menus,
and an ISO too large for CI. `docs/ARCHITECTURE.md` has the full reasoning.

Two constraints follow from that, and both are easy to break:

- **The core must stay offline-useful.** Its whole justification is that a
  live USB works air-gapped. Do not thin it out on size grounds without
  saying what offline capability is being given up.
- **Everything past the core belongs to Sarina**, including generating
  `.desktop` entries and start-menu categories for newly installed tools.
  Neither capability exists yet. Do not add tools to `packages.x86_64` to
  work around that gap — it makes the eventual agent-side work harder to
  justify and quietly re-creates the problem the categories caused.

Verify any change to the package list by resolving it against live repos on
an Arch host with `[blackarch]` and `[labyrinth]` configured — this catches
conflicts and provider menus that CI would only surface hours later, or not at
all:

```bash
mapfile -t PKGS < <(sed '/^[[:blank:]]*#.*/d;s/#.*//;/^[[:blank:]]*$/d' profile/packages.x86_64)
pacman -Sp --print-format '%r/%n' "${PKGS[@]}"   # must exit 0, print no provider menu
```

Adding or removing whole tool *categories* is a product decision for the
maintainer, not a build fix to apply unilaterally — it changes what the
distro is.

## Working alongside other Claude instances

Other instances work on the sibling projects in this ecosystem — chiefly
**Sarina** (`Artifex-Technologies/Sarina`, installed here as `sarina-git` from
`[labyrinth]`). They
have repo access. This section is the coordination surface: keep it a
**contract**, not a message log. Anything that is a passing note belongs in a
commit message or the PR, not here — CLAUDE.md loads into every session, so
churn here costs every future session context.

If you change something on this list, say so in your commit message using the
word `CONTRACT:` so the other side can grep for it.

### What LykanOS consumes from Sarina

The `sarina-git` package from `[labyrinth]`, built from Labyrinth's recipe
(`packages/pkgbuilds/sarina-git` in Labyrinth_OS); its packaging contract with
the Sarina repo is kept there, not here. What LykanOS itself relies on:

- the package name `sarina-git` in `profile/packages.x86_64`, and the entry
  points `sarina`, `sarina-gui` and `sarina-service` on `PATH`;
- the `sarina` account name, reserved in `lykanos-installer.conf`
  (`reserved_logins`) so the installer never offers it;
- Heimdall's `contained` file and release command (`lykanos-security.conf`),
  which act on Sarina.

The package also installs a Yggdrasil `assistant` integration
(`/usr/share/yggdrasil/integrations/sarina.conf`) whose programs
(`/usr/local/bin/labyrinth-sarina-*`) are part of Labyrinth's image, not of the
package, so on LykanOS they are missing (`docs/DESKTOP.md`, "Sarina in the
desktop").

**Licensing.** The package is MPL-2.0 (its `LICENSE` and `license` field).
Sarina's git history still contains code from before its clean-room rewrite
(Sarina's `NOTICE.md` and `docs/PROVENANCE.md`). An ISO ships the built
package, not the history, but this is not legal advice: confirm with the owner
before a publicly distributed ISO.

### What LykanOS needs Sarina to provide (not built yet)

The ISO deliberately ships a small tool core and defers the rest. These are
the capabilities that gap depends on — see `docs/ARCHITECTURE.md`:

1. **Install-time tool selection** — ask what the machine is for, install
   accordingly, rather than a category guess baked into the ISO.
2. **Categorisation** — generate `.desktop` entries and start-menu
   categories for tools that drop bare binaries into `/usr/bin`. Measured on
   the 2026-09-12 ISO (the KDE one), so this is the real shape of the problem,
   not a guess:

   | | Count |
   | --- | --- |
   | `.desktop` files in the image | 181 |
   | …declaring a standard `Security` category | 2 |
   | Of 15 shipped security tools sampled, ones with any menu entry | 2 |

   Two distinct jobs, not one. **Generating** entries: 13 of those 15 tools
   (metasploit, sqlmap, nmap, radare2, john, hashcat, hydra, aircrack-ng,
   bettercap, nikto, binwalk …) ship no `.desktop` file at all, so on the
   desktop they are invisible — you must already know they exist and open a
   terminal. **Remapping** entries: BlackArch packages that do ship one use
   non-standard `X-BlackArch-*` categories (`burpsuite` declares
   `X-BlackArch-Fuzzer;X-BlackArch-Proxy;X-BlackArch-Scanner;X-BlackArch-Webapp`).
   Yggdrasil's start menu makes its folders from the freedesktop main
   categories only (`packages/panel/src/apps.rs` in Yggdrasil), so those land
   in its **Other** folder; it has no Security folder.

   A useful target is an `X-BlackArch-*` → freedesktop category mapping plus
   synthesised entries that launch CLI tools in a terminal (Ghostty). The image's
   icon theme is Liki (`liki-icon-theme`), so generated entries should reference
   icon names it provides, or they will show a generic fallback.

   **Wired 2026-11-07:** `/usr/local/bin/lykanos-tool-menu`, run by
   `etc/pacman.d/hooks/90-lykanos-tool-menu.hook` after every pacman transaction
   (and while the ISO is built), runs `sarina desktop-audit --apply
   --only-category Security --target-dir /usr/local/share/applications` (the flag
   is Sarina c90f1ae). Yggdrasil's start menu has a Security folder and opens
   `Terminal=true` entries in a terminal with a shell left open (Yggdrasil
   7236d54). Remapping existing `X-BlackArch-*` entries and icons are not done.

3. **Data analysis** — the reason the agent is in the OS at all: making sense
   of scan and capture output in place.
4. **`blackarch_compat` against a real Docker daemon** — currently
   unit-tested only. It is the safety check behind (1); until it has actually
   run, treat any category the agent proposes as unvalidated.
5. **Crash diagnosis** — a `systemd-coredump` watcher that raises a "Process
   crashed" notification and hands the PID, binary and signal to Sarina with a
   skill describing how to investigate (coredump, Proton/Wine log, `journalctl`,
   GPU driver state). The design follows Omarchy's `omarchy-crash-watch` /
   `omarchy-agent-crash`; see `docs/DESKTOP.md`. Sarina proposes, a human applies.
6. **Proton VPN lifecycle** — install `proton-vpn-gtk-app` (Arch `extra`) on
   request, and after `pacman -Syu` report what changed for the VPN. The kill
   switch must live in nftables or NetworkManager, *not* in Sarina, so a crashed
   or injected agent cannot leave the VPN open. Network-config changes are
   proposals only.
7. **Agent manager** — read the other agents' usage records for a unified
   view, and only later launch or supervise other agents, proposal-first with
   human approval and Heimdall watching (an unsupervised launcher that starts
   agents in auto-approve modes is a privilege-escalation path). **Built on the
   Sarina side** as `sarina fleet-usage`, `fleet-status` and `fleet-launch`
   (Sarina's `docs/ECOSYSTEM.md` §1). The display was to be Omarchy's agents
   panel, which is not coming now that the desktop is Yggdrasil. Yggdrasil's
   way to show it would be a `bar-item` integration file (Yggdrasil's
   `docs/INTEGRATIONS.md`). See `docs/DESKTOP.md`.

### New Sarina versions

A new Sarina reaches LykanOS when Labyrinth's publish puts a new `sarina-git`
into `[labyrinth]`: the next ISO build and `pacman -Syu` pick it up. Nothing in
this repo changes for it.
