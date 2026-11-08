# Architecture

## Layers

1. **Base**: Arch Linux, built via [archiso](https://github.com/archlinux/archiso).
   `profile/` is a fork of archiso's `releng` profile (`profiledef.sh`,
   `packages.x86_64`, `pacman.conf`), kept close to upstream so Arch's own
   install/live-boot support (hardware drivers, `archinstall`, rescue tools)
   isn't quietly lost to hand-trimming.
2. **Security tooling**: [BlackArch](https://github.com/BlackArch/blackarch),
   added as a repo (`[blackarch]` in `profile/pacman.conf`) rather than
   forked. The ISO bakes in only a small hand-picked core (26 tools and
   `blackarch-keyring` in `profile/packages.x86_64`); everything beyond it is
   installed after the fact, agent-assisted, by Sarina. See "Tool delivery"
   below for why.
3. **Desktop**: [Yggdrasil](https://github.com/Artifex-Technologies/Yggdrasil)
   on Hyprland, the same desktop as Labyrinth OS, with greetd and Yggdrasil's
   greeter as the login screen. Its packages come from Labyrinth OS's signed
   package repository (`[labyrinth]`, last in both `pacman.conf` files,
   `SigLevel = Required`), not from copies in this repo; LykanOS adds
   integration files (`profile/airootfs/usr/share/yggdrasil/integrations/`), its
   look and the start-menu hook for the security tools.
   NetworkManager, BlueZ and PipeWire sit under it (`profile/packages.x86_64`
   layers these on top of releng's more CLI/live-boot-oriented networking and
   audio stack), and since 2026-11-07 the laptop and graphics set from
   Labyrinth's base list (Vulkan for Intel and AMD graphics, Intel video
   decoding, the sound mixer setups, power modes). Until 2026-11-07 the desktop
   was KDE Plasma with SDDM. Details and what is unverified are in
   [DESKTOP.md](DESKTOP.md).
4. **Installer**: Yggdrasil's installer (`yggdrasil-installer`, with its
   install program `yggdrasil-install-system`), shared with Labyrinth OS and
   configured by `lykanos-installer.conf`; LykanOS's own clean-up is
   `lykanos-install-cleanup`. A graphical installer suits a live USB better
   than `archinstall`'s TUI. It installs by copying the live squashfs rather
   than running `pacstrap`, which is what makes it quick and lets it work
   offline. It needs UEFI. How and why, and what is unverified, is in
   [INSTALLER.md](INSTALLER.md). `archinstall` stays in `packages.x86_64`
   (from releng) as a CLI/scripted alternative. Until 2026-11-07 the installer
   was Calamares.
5. **Security monitor**: [Heimdall](https://github.com/Artifex-Technologies/Heimdall)
   (`heimdall-git` from `[labyrinth]`), shared with Labyrinth OS, with ClamAV;
   installed but off (`/etc/heimdall/defaults-off`, 2026-11-07). Its rules for
   the security tools are in force (`/etc/heimdall/security-tools`, Heimdall's
   `docs/TOOL_RULES.md`); it stays off until they are tested with the tools in a
   LykanOS VM. `ufw` is installed but off. See [DESKTOP.md](DESKTOP.md).
6. **AI agent**: [Sarina](https://github.com/Artifex-Technologies/Sarina),
   installed as `sarina-git` from `[labyrinth]`, the same package Labyrinth OS
   installs. Nothing is built in this repo: until 2026-11-07 `sarina-git` was
   built into a local `[niflheim]` repo from `packages/sarina`, a copy of
   Sarina removed on 2026-10-08.

## Tool delivery: a small baked-in core, then Sarina

Kali ships a curated default set (`kali-linux-default`) rather than every tool
in its repos, to keep the base system usable. LykanOS originally copied that
shape by baking in fifteen `blackarch-*` category groups. **That approach was
abandoned**; the ISO now ships a small hand-picked core and defers the rest to
Sarina at install time.

### Why the categories were dropped

The category list was never validated, and when CI first got far enough to try
it, it failed in three separate ways:

- **Unresolvable dependencies.** `yinjector`, `limelighter` and `thefatrat`
  need `lib32-*` packages, which forced `[multilib]` on. Enabling it fixed
  those three and said nothing about the thousands of packages behind them.
- **Silent provider roulette.** Six virtual dependencies had multiple
  providers, so pacstrap printed a numbered menu for each — `tessdata` alone
  offered 128 — and, being non-interactive, took option 1 every time. That is
  how the ISO nearly shipped with Afrikaans OCR data.
- **Size.** The resolved set did not fit on a CI runner and would have made a
  multi-tens-of-GB ISO. It killed the GitHub runner outright, which uploads no
  log at all when it dies, making the failure invisible.

Behind all three is one problem: nobody had decided what those ~3,000 packages
were *for*. A category group is a guess about what a user wants, made by
someone who has never met them.

### What ships on the ISO now

A hand-picked core covering recon, capture, wireless, password attacks, web,
exploitation, reversing and forensics. Named packages, not groups — the size
is predictable and the conflict surface is nearly nil. Verified against live
repos: 837 packages resolved including dependencies, no provider menus, no
conflicts, ~3.2 GiB compressed (measured with the earlier KDE desktop). The ISO
with Yggdrasil, built by CI on 2026-11-08: 874 packages including dependencies,
4.2 GiB with zstd, and one provider menu left unpinned (`java-runtime>=17`;
CLAUDE.md, "Pin providers explicitly").

The core exists for one reason worth stating plainly: **the live session has
to be useful with no network.** Air-gapped client sites, SCIFs and offline
forensics are exactly where a live USB earns its keep, and they are the one
thing a download-on-demand model cannot serve. `blackarch-keyring` ships too,
so the installed system trusts `[blackarch]` and can expand later.

### What Sarina owns

Everything past the core:

- **Guided install-time selection** — asking what the machine is for and
  installing accordingly, instead of guessing on the user's behalf.
- **Categorisation.** BlackArch packages drop bare binaries into `/usr/bin`
  with no menu entry — the exact problem Kali's curated menus solve. Sarina
  generates `.desktop` entries and start-menu categories as tools are added,
  so the organisation is maintained rather than hand-written here.

Guided selection does not exist yet. Categorisation exists in a first form
since 2026-11-07: `lykanos-tool-menu`, run by a pacman hook, has Sarina's
`desktop-audit` write a start-menu entry for each installed security tool
([DESKTOP.md](DESKTOP.md), "The tools in the start menu"); remapping the
`X-BlackArch-*` categories of entries that tools ship themselves is not done.
Until selection exists, the core is all the distro ships, and that is a
deliberate floor rather than a placeholder — see "Open questions".

## Validating additions: `blackarch_compat`

The Sarina repository ships a purpose-built harness at
`blackarch_compat/`: it installs one `blackarch-*` category
(or an explicit combination, with `--combo`) at a time inside a disposable
Docker container and records whether the install goes cleanly.

```bash
# in a checkout of the Sarina repository
python3 -m blackarch_compat.run_tests --all -o report.json --markdown report.md
```

Its role has shifted with the change above. It is no longer a gate on the ISO
package list — that list is now small enough to verify directly:

```bash
# on an Arch host with [blackarch] and [labyrinth] configured
mapfile -t PKGS < <(sed '/^[[:blank:]]*#.*/d;s/#.*//;/^[[:blank:]]*$/d' profile/packages.x86_64)
pacman -Sp --print-format '%r/%n' "${PKGS[@]}"
```

Instead it becomes the safety check behind *Sarina's* install-time
suggestions: before the agent offers a category, it should know that category
installs cleanly alongside what is already present. As of this writing the
harness is unit-tested but **has not been run against a real Docker daemon**
(see its own README's Status section).

What it catches: install-time conflicts (`pacman -S` failures, `pacman -Qk`
file-manifest mismatches). What it doesn't: runtime conflicts between tools
(port/config/venv collisions), or anything that only shows up on the actual
built ISO — for that, a local [archiso](https://github.com/archlinux/archiso)
checkout's `scripts/run_archiso.sh` boots a built ISO in QEMU.

## Live session

The live medium signs a passwordless `live` user in to the Yggdrasil session:
greetd reads `etc/greetd/live.toml` there (through the drop-in
`greetd.service.d/live.conf`), which adds an `initial_session` to the normal
login screen. Everything in this section is live-medium only: the installer
strips it from the installed system (`lykanos-install-cleanup`, see
[INSTALLER.md](INSTALLER.md)). Getting here took three things that are each easy
to get wrong:

**The user is created by `sysusers.d`, not by shipping `/etc/passwd`.**
archiso's releng profile ships a two-line `airootfs/etc/passwd` containing only
root, and that is safe *for releng* because nothing in its package set needs a
service account. It is not safe here. mkarchiso copies `airootfs/` over the
pacstrapped root and never re-runs `systemd-sysusers`, so a hand-written
`passwd` silently deletes every account the image already has — including
greetd's `greeter`, without which the login screen cannot start.
`airootfs/etc/sysusers.d/live.conf` sidesteps this: `systemd-sysusers.service`
runs at boot, before `sysinit.target`, and adds the account (in `wheel`,
`video`, `audio` and `input`) to the real `/etc/passwd`.

**`systemd-firstboot` has to be masked.** mkarchiso deliberately writes
`/etc/machine-id` as the literal string `uninitialized` so systemd generates a
fresh id per boot. That also makes `ConditionFirstBoot=yes` true, so
`systemd-firstboot` runs and interactively asks for timezone, locale, hostname
and root password before anything graphical starts — on a live ISO, a dead end.
Shipping a real machine-id does not help, because mkarchiso overwrites it. So
`etc/systemd/system/systemd-firstboot.service` is a link to `/dev/null`, made by
`scripts/build.sh` in the work copy of the profile together with the other
service links (the default target, `display-manager.service` -> greetd,
NetworkManager, Bluetooth, `pacman-init.service`). They are not kept in git:
a checkout without symlinks turns a committed link into a text file holding its
target, which systemd reads as a broken unit, and until 2026-11-07 four of
them were such files.

**Passwordless means every auth prompt must be removed, not just the login
one.** The account has no password, so anything that asks for one strands the
user. That is why the profile also ships `sudoers.d` and a polkit rule granting
`wheel` unprompted access. Both are live-medium only: the install program
writes the installed system's own sudoers file, and `lykanos-install-cleanup`
deletes the live files so none of this reaches it. That script keeps its own
list of these files, so a new live-only file has to be added there too.
The screen does not lock by itself (`yggdrasil-idle` only observes unless its
config says `mode = act`, and the live account has none), but a manual lock
(Super + L) would strand the live user; see [DESKTOP.md](DESKTOP.md).

**pacman's keyring is made at start-up.** mkarchiso builds the image without a
keyring, so `pacman-init.service` creates one on the live medium and puts in
the archlinux, blackarch and labyrinth keys. Without the Labyrinth key, `pacman`
on the live medium could not read `[labyrinth]`.

Files for the live user go in `airootfs/home/live/` (today only the installer
launcher in `Desktop/`), chowned recursively by a `file_permissions` entry whose
path ends in `/` (mkarchiso only recurses for those). The uid is hardcoded to
1000 to match the pin in `sysusers.d/live.conf`, because the account does not
exist at build time.

## Package flow

```text
Yggdrasil, Heimdall and Sarina repos
        │  recipes in Yggdrasil's packages/pkgbuilds (the yggdrasil-* packages, saga-md, liki-icon-theme)
        │  and Labyrinth_OS's packages/pkgbuilds (sarina-git, heimdall-git, labyrinth-keyring);
        │  built and signed by Labyrinth's publish
        ▼
[labyrinth]   (GitHub release "packages" of Labyrinth_OS; Labyrinth's docs/PACKAGE_REPO.md)
        │
        ▼  scripts/build.sh → mkarchiso, with Arch and [blackarch] (profile/pacman.conf)
out/lykanos-<date>-x86_64.iso   (in CI: the workflow artifact lykanos-iso)
```

Nothing is built in this repository. A new Sarina, desktop or Heimdall reaches
the next ISO, and installed systems through `pacman -Syu`, once Labyrinth's
publish has put it in `[labyrinth]`. A package added to `packages.x86_64`
before the publish carries it stops the build (CLAUDE.md). The `sarina-git`
recipe is Labyrinth's `packages/pkgbuilds/sarina-git`; the AUR-facing PKGBUILD that builds from
tagged release tarballs lives in the Sarina repo itself, at
`packaging/arch/PKGBUILD` (`pkgname=sarina` there), and is the source of truth
for the systemd unit, sysusers snippet and env file both install.

## Sarina's role in the distro

Beyond running as a packaged, on-system service (`sarina-service`, from
`packaging/arch/` upstream), the intent is for Sarina to take an active
hand in maintaining LykanOS itself over time — not just be an app that
happens to ship on it:

- **Tool organization**: raw BlackArch packages tend to drop binaries into
  `/usr/bin` with no menu entry or categorization (the exact "installs
  things wherever" problem Kali's curated menus solve). Sarina is meant to
  help generate/maintain `.desktop` entries and start-menu categories for
  newly added tools as the curated `blackarch-*` selection evolves, rather
  than that being purely a hand-maintained list here.
- **Keeping the system current, safely**: helping evaluate `pacman -Syu` /
  BlackArch updates for what's safe to apply automatically vs. what needs a
  human look (ABI breaks, config-file conflicts, a tool category that failed
  its last `blackarch_compat` run), with an explicit bias toward secure,
  functional, and efficient over just "newest."

The first exists in a first form since 2026-11-07: Sarina's `desktop-audit`
command, which `lykanos-tool-menu` runs from a pacman hook. The second is
direction, not implementation. Both live in the Sarina repo itself, with this
repo consuming the surface they expose (for the menu, a CLI command run by a
pacman hook; for maintenance, a CLI command, a systemd timer, a Yggdrasil
integration file, etc. — TBD).

## Open questions / not yet decided

- Whether the hand-picked core is the right size and shape. It is a floor,
  chosen for offline usefulness, not a considered curation — revisit once
  Sarina's install-time selection exists and it is clear what the core has to
  cover on its own.
- Sequencing: the lean ISO assumes Sarina will deliver tools post-install, and
  that capability does not exist yet. Until it does, LykanOS ships less
  tooling than it did as a concept. That is an accepted, temporary cost.
- The installer has never been run on LykanOS (see [INSTALLER.md](INSTALLER.md)).
- LykanOS's own look: accent, wallpaper, logo and splash exist since
  2026-11-07 (DESKTOP.md, "The look"); no boot menu theme or boot animation yet.
- Heimdall's tool rules: built and in force for the 29 tools in its catalog
  (2026-11-07); Heimdall stays off until they are tested with the tools in a
  LykanOS VM. Not covered yet: Wi-Fi monitor mode, payload files, C2 servers.
  And a firewall profile for hostile networks such as a hacker convention.
- Concrete shape of Sarina's on-system maintenance role (see above) — CLI
  tool, systemd timer, a Yggdrasil integration, or some combination.
