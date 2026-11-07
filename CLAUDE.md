# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Push every commit as you make it; don't ask me first. Before pushing, check the diff names no host name, network address, SSID
or MAC address.

Since 2026-11-07 the desktop is Yggdrasil on Hyprland, with greetd and Yggdrasil's installer, the same as Labyrinth OS's
(until then KDE Plasma, SDDM and Calamares). `docs/YGGDRASIL_MOVE.md` records the move; its step 6, building the ISO and
checking it in a VM, is still open.

## What this repo is

Niflheim OS is an Arch-based security distribution. This repo contains **no
application code** — it is an [archiso](https://github.com/archlinux/archiso)
profile plus the packaging and build glue that turns it into a bootable ISO.
Sarina lives in a separate repo, vendored here as a submodule. The desktop
(Yggdrasil), Heimdall and `labyrinth-keyring` are packages from Labyrinth OS's
signed repository (`[labyrinth]`); their code lives in the Yggdrasil, Heimdall
and Labyrinth_OS repos, not here.

Read `docs/ARCHITECTURE.md` before making design decisions; it records *why*
the current structure exists (a small baked-in tool core over BlackArch
category groups, the desktop as packages rather than copies, a graphical
installer over `archinstall`, etc.). `docs/DESKTOP.md` covers the Yggdrasil
desktop and what Niflheim adds to it, `docs/INSTALLER.md` the install flow;
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

Two stages, and the root/non-root split is mandatory, not stylistic:
`makepkg` refuses to run as root, and `mkarchiso` requires it.

```bash
# 1. Build local packages (sarina-git) into repo/x86_64/
#    MUST be a regular user.
./scripts/build-local-packages.sh

# 2. Build the ISO. MUST be root. Reads repo/x86_64/ from step 1.
sudo ./scripts/build.sh

# Advance the Sarina submodule and stage the pointer bump
./scripts/update-sarina.sh
```

`scripts/build.sh` also installs BlackArch's key (BlackArch's `strap.sh`),
trusts the Labyrinth package key by its full fingerprint
(`C9CE784C8BD9B661042075972D3BD4B1E7CD07A5`: fetched, fingerprint checked, then
`pacman-key --lsign-key`) and makes the systemd service links in the work copy
of the profile (see below). It honours `NIFLHEIM_WORK_DIR` and
`NIFLHEIM_OUT_DIR` for build hosts where `work/` and `out/` must sit on a
different filesystem; CI uses these to move the build off the runner's small
root disk.

There is no test suite, linter, or build step for this repo itself. Sarina has
its own (`packages/sarina`, own CLAUDE.md) — do not run or modify those from
here.

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

**`profile/pacman.conf` contains a `@@NIFLHEIM_REPO_DIR@@` placeholder.** It is
deliberate — `scripts/build.sh` substitutes an absolute path into a *work copy*
so the checked-in file stays portable. Do not "fix" it to a real path.

**There are two `pacman.conf` files, and `[labyrinth]` is last in both.**
`profile/pacman.conf` is for the build (with `[blackarch]` and the local
`[niflheim]`); `profile/airootfs/etc/pacman.conf` is the live medium's and the
installed system's (Arch's repos and `[labyrinth]` only). `[labyrinth]` has
`SigLevel = Required` and is last on purpose: a package of the same name above
it wins (`sarina-git` from `[niflheim]` at build time). Do not vendor copies of
the `yggdrasil-*` packages, `heimdall-git` or `labyrinth-keyring` into
`packages/pkgbuilds`: they come from `[labyrinth]`, so updates reach Niflheim
and Labyrinth the same day.

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
`unsquashfs` on `airootfs.sfs` and then Niflheim's clean-up inside the new
system (`niflheim-installer.conf` names it; details in `docs/INSTALLER.md`).
Traps, each of which produces an install that "succeeds" and then fails later:

- mkarchiso deletes everything in `/boot` from the image, so the target has no
  kernel, and the image's initramfs preset and `mkinitcpio.conf.d/archiso.conf`
  are archiso's. `niflheim-install-cleanup` restores both. Do not remove it.
- That script keeps its own list of live-only files (sudoers, polkit, greetd's
  `live.toml` and its drop-in, the installer launcher and integration file,
  ...). Add any new live-only file to it as well as to `airootfs/`; nothing
  checks the two against each other.
- The program finds the image at `/run/archiso/bootmnt/<ID>/x86_64/airootfs.sfs`,
  with `ID` from `/etc/os-release` (`niflheim`), which must equal `install_dir`
  in `profiledef.sh` (not archiso's `arch`). Change one and you must change the
  other, or set `medium` in `niflheim-installer.conf`.
- The program installs on UEFI computers only. The medium also boots BIOS
  (syslinux); there only `archinstall` installs.
- Changes to the install steps themselves belong in Yggdrasil
  (`packages/installer`, its `docs/INSTALLER.md`); Niflheim's part is the
  integration file and the clean-up. Nothing of it has been run on Niflheim
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
- **Submodule token.** `actions/checkout` uses the `SARINA_REPO_TOKEN` secret
  when it is set (Contents: read on *both* repos, since it replaces the token
  for the whole checkout), from when Sarina was a private repo. Sarina is
  public now, so the default token is enough.

**No run has produced an ISO since the repository moved here (2026-10-08).**
Every run stops at "Build local packages": the scripts are stored without the
executable bit (`sudo: ./scripts/build-local-packages.sh: command not found`),
and the tree has no gitlink for `packages/sarina` (only `.gitmodules`), so the
script would stop next at "submodule looks empty".

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
mapfile -t PKGS < <(sed '/^[[:blank:]]*#.*/d;s/#.*//;/^[[:blank:]]*$/d' profile/packages.x86_64 | grep -vx sarina-git)
pacman -Sp --print-format '%r/%n' "${PKGS[@]}"   # must exit 0, print no provider menu
```

Adding or removing whole tool *categories* is a product decision for the
maintainer, not a build fix to apply unilaterally — it changes what the
distro is.

## Working alongside other Claude instances

Other instances work on the sibling projects in this ecosystem — chiefly
**Sarina** (`Artifex-Technologies/Sarina`, vendored here as `packages/sarina`). They
have repo access. This section is the coordination surface: keep it a
**contract**, not a message log. Anything that is a passing note belongs in a
commit message or the PR, not here — CLAUDE.md loads into every session, so
churn here costs every future session context.

If you change something on this list, say so in your commit message using the
word `CONTRACT:` so the other side can grep for it.

### What Niflheim consumes from Sarina

`packages/pkgbuilds/sarina-git/PKGBUILD` builds the submodule directly. It
will break if any of these move or disappear — verified present as of the
`9508cc0` submodule pointer (the Go rewrite; `build()`/`package()` run on Arch):

| Path in the Sarina repo | Used for |
| --- | --- |
| `go/go.mod`, `go/cmd/sarina` | `go build` of the single static binary |
| `python/sarina_helpers/pyproject.toml` | `python -m build --wheel --no-isolation` for the optional ML/voice helper |
| `packaging/arch/sarina-service.service` | installed to `/usr/lib/systemd/system/` |
| `packaging/arch/sarina.sysusers` | installed to `/usr/lib/sysusers.d/sarina.conf` |
| `packaging/arch/service.env.example` | installed to `/etc/sarina/` |
| `README.md` | installed to `/usr/share/doc/sarina-git/` |
| `LICENSE` | installed to `/usr/share/licenses/sarina-git/LICENSE` |

Runtime dependency declared in that PKGBUILD is only `glibc` (the Go binary is
static); `python` and `python-onnxruntime` are optional dependencies of the
helper. **Keep these in sync with Sarina's real dependencies**, and every one
must exist in Arch's repos. A dependency that is PyPI-only needs a PKGBUILD of
its own here first. (The old Python build needed `python-fastapi`, `uvicorn`
— no `python-` prefix on Arch — and `python-pydantic`; they are gone.)

Entry points the ISO expects on `PATH`: `sarina`, `sarina-gui`,
`sarina-service` — now one binary plus two symlinks to it (the binary picks its
command from the name it was started as).

**Licensing caveat (updated 2026-10-06).** As of the `9508cc0` submodule pointer
the Sarina working tree no longer contains the unlicensed upstream code: the
core was rewritten clean-room in Go and the derived Python deleted (see
`packages/sarina/NOTICE.md` and `docs/PROVENANCE.md`; a word-sequence audit found
0.01% overlap). Sarina is now MIT-licensed (owner decision 2026-10-06, installed
by the PKGBUILD as `/usr/share/licenses/sarina-git/LICENSE`). Still open and owned
by the Sarina owner: Sarina's git history still contains the old code. An ISO ships the built
`sarina-git` package, not the history, but this is not legal advice: confirm with
the owner before a publicly distributed ISO, and note that anyone fetching the
submodule gets the history. A private/internal ISO is unaffected.

### What Niflheim needs Sarina to provide (not built yet)

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
   panel, which is not coming now that the desktop is Yggdrasil; the
   `sarina-git` PKGBUILD still installs the Omarchy collector
   `/usr/share/omarchy/bin/omarchy-agent-usage-sarina`, which nothing on the
   image runs. Yggdrasil's way to show it would be a `bar-item` integration
   file (Yggdrasil's `docs/INTEGRATIONS.md`). See `docs/DESKTOP.md`.

### Advancing the submodule

`./scripts/update-sarina.sh` moves the pointer and stages it. After bumping,
re-check the table above before committing — a green ISO build is the only
thing that proves the packaging contract still holds.
