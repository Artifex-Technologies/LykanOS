# Desktop: Yggdrasil

LykanOS's desktop is [Yggdrasil](https://github.com/Artifex-Technologies/Yggdrasil), the desktop environment written in
Rust that Labyrinth OS uses, running on Hyprland. I decided on 2026-11-07 to use the same desktop as Labyrinth OS rather
than keep one of LykanOS's own; the move is recorded in [YGGDRASIL_MOVE.md](YGGDRASIL_MOVE.md). Until 2026-11-07 the
desktop was KDE Plasma (SDDM, the MacTahoe theme, Kvantum) with an Omarchy-style Hyprland session and two Waybar looks
beside it. Those files are gone from the profile.

This page says what LykanOS takes from Yggdrasil, where the packages come from and what LykanOS adds. For the desktop
itself, Yggdrasil's own documentation is the reference: its README (the programs and the keyboard shortcuts),
`docs/INTEGRATIONS.md` and `docs/INSTALLER.md`.

## Status

| Piece | State |
| --- | --- |
| Package list | Installs in full: the CI build of 2026-11-08 resolved it against Arch, BlackArch and `[labyrinth]` (874 packages with dependencies). One provider choice is not pinned yet (CLAUDE.md, "Pin providers explicitly") |
| ISO with Yggdrasil | Built by CI since 2026-11-07 (7d2bfe5). The latest is `lykanos-2026.11.08-x86_64.iso` (27f3064), 4.2 GiB; see the README's Status |
| Live session, login screen, Settings, Agora, installer | **Not booted in a VM yet** (step 6 of [YGGDRASIL_MOVE.md](YGGDRASIL_MOVE.md)) |
| The desktop packages themselves | Run on Labyrinth OS, which installs the same ones; not yet with LykanOS |
| Real hardware (a GPU, a laptop) | Not tried with LykanOS |
| Proton VPN, Sarina crash diagnosis, agent manager | Contracts only; see below |
| Tailscale / Headscale | Optional, not installed |

## What is on the image

From `profile/packages.x86_64`:

- **The session:** `yggdrasil-session` (Yggdrasil's Hyprland configuration, window rules and keys) on Arch's `hyprland`,
  with `uwsm`, `xdg-desktop-portal-hyprland`, `xdg-desktop-portal-gtk` and `hyprpolkitagent`.
- **Yggdrasil's programs:** `yggdrasil-panel` (top bar, dock, start menu, notifications), `yggdrasil-settings`,
  `yggdrasil-minos` (the file manager, and through `yggdrasil-portal` every program's open and save dialog),
  `yggdrasil-agora` (the app store), `yggdrasil-lock` (lock screen and the login screen's greeter), `yggdrasil-idle`,
  `yggdrasil-polkit` (the password box), `yggdrasil-keyring-sync`, `yggdrasil-xmove`, `yggdrasil-clipboard`,
  `yggdrasil-colorpicker`, `yggdrasil-calculator`, `yggdrasil-taskmanager` (the Task Manager, Ctrl + Shift + Esc;
  2026-11-07), `saga-md` (Saga MD, the Markdown reader: Markdown and text files formatted, or their source to read and
  edit; 2026-11-08), `yggdrasil-spell` (with `hunspell-en_us`), `yggdrasil-installer`, `yggdrasil-theme` (GTK and Qt
  look, the pointer) and `liki-icon-theme`.
- **Terminal:** Ghostty (Super + Return).
- **Browser:** Firefox.
- **Saved passwords:** GNOME Keyring (`gnome-keyring`), the same choice Yggdrasil made; I kept it on 2026-11-07.
- **Wi-Fi, Bluetooth and sound:** Yggdrasil's Settings, over NetworkManager, BlueZ (`bluez`, `bluez-utils`) and PipeWire.
- **Laptops and graphics** (18206cd, 2026-11-07: my first LykanOS laptop is an Intel one; the same set as Labyrinth OS's
  base list): `vulkan-intel` and `vulkan-radeon` (Vulkan for Intel and AMD graphics), `intel-media-driver` (Intel
  video decoding), `alsa-ucm-conf` (the sound mixer setups: with releng's `sof-firmware`, most Intel laptops have no sound
  without both) and `power-profiles-daemon` (the power modes in Yggdrasil's Settings and battery applet).
- **Small tools the session uses:** `grim`, `slurp`, `wl-clipboard`, `brightnessctl`, `playerctl`, `libnotify`,
  `noto-fonts-emoji`, and `btop` (running programs, in a terminal; Ctrl + Shift + Esc opened it until the Task Manager
  took that key on 2026-11-07).

Saga MD is not made the default program for `.md` and `.txt` files here: Labyrinth's image does that in
`/etc/xdg/mimeapps.list`, and LykanOS's image has no such file. Which program opens them is not checked yet.

Yggdrasil's other programs (the video player `yggdrasil-player`, the weather `yggdrasil-weather`, the spell-check word
packs and the store's apps) are not on the image. `[labyrinth]` has them, so pacman installs them; LykanOS's Agora catalog
does not list them (see "Agora on LykanOS").

## Login screen

greetd runs Yggdrasil's greeter inside `cage`, a small compositor, on a console of its own (VT 7):
`profile/airootfs/etc/greetd/config.toml`. The greeter account (`greeter`) comes from the greetd package.

On the live medium a drop-in, `etc/systemd/system/greetd.service.d/live.conf`, makes greetd read `etc/greetd/live.toml`
instead. That file is the same login screen plus an `initial_session`: once per start-up the live user is signed in to
`yggdrasil-session` without a password. After signing out, the normal login screen comes. `lykanos-install-cleanup`
deletes both files, so an installed system starts at the login screen.

`display-manager.service` (pointing at `greetd.service`) is one of the links `scripts/build.sh` makes in the work copy of
the profile; see CLAUDE.md for why they are not in git.

## Where the packages come from

| Packages | Repository |
| --- | --- |
| `yggdrasil-*`, `saga-md`, `liki-icon-theme`, `heimdall-git`, `sarina-git`, `labyrinth-keyring` (and, not on the image, `argus-git` for Agora and `limbo-git`) | `[labyrinth]`: Labyrinth OS's signed package repository |
| `hyprland`, `greetd`, `cage`, `ghostty`, `gnome-keyring`, `btop`, `ufw`, `clamav` and the rest | Arch (`core`, `extra`, `multilib`) |
| the security tools | Arch and `[blackarch]` (build only; see [ARCHITECTURE.md](ARCHITECTURE.md)) |

Nothing is built in this repository. Until 2026-11-07 `sarina-git` was built into a local package repository from
`packages/sarina`, a copy of Sarina that had been removed on 2026-10-08.

`[labyrinth]` is the GitHub release `packages` of `Artifex-Technologies/Labyrinth_OS` (Labyrinth's
`docs/PACKAGE_REPO.md`). Its packages are built for any 64-bit x86 CPU, not for the build machine's. It is listed last,
with `SigLevel = Required`, in both pacman configurations:

- `profile/pacman.conf`, used by `mkarchiso` to build the image;
- `profile/airootfs/etc/pacman.conf`, the live medium's and the installed system's. It lists Arch's repositories and
  `[labyrinth]` only: no `[blackarch]` (its keyring is installed so it can be trusted later).

Being last means a package of the same name in Arch's repositories (or `[blackarch]` at build time) always wins, so
`[labyrinth]` can add packages but never replace the base system.

The signing key is `C9CE784C8BD9B661042075972D3BD4B1E7CD07A5`:

- **On the build host**, `scripts/build.sh` downloads `labyrinth.gpg` from Labyrinth's repository, adds it, checks that it
  has that full fingerprint and only then signs it locally (`pacman-key --lsign-key`).
- **On the live medium**, `pacman-init.service` creates pacman's keyring at start-up and puts in the archlinux,
  blackarch and labyrinth keys (from `archlinux-keyring`, `blackarch-keyring` and `labyrinth-keyring`).
- **On the installed system**, `lykanos-install-cleanup` does the same during the install; `pacman-init.service` stays as
  a safety net and runs only when there is no keyring.

So `pacman -Syu` brings desktop, Heimdall and Sarina updates the same day Labyrinth gets them.

## What LykanOS adds: integration files

Yggdrasil never names an operating system; it reads integration files (Yggdrasil's `docs/INTEGRATIONS.md`). LykanOS's
are in `profile/airootfs/usr/share/yggdrasil/integrations/`:

| File | Kind | What it does |
| --- | --- | --- |
| `lykanos-branding.conf` | `branding` | The accent colour `#437569`, the wallpaper and the splash logo (see "The look" below). The name and logo come from `/etc/os-release` |
| `lykanos-installer.conf` | `installer` | Points the installer window at `yggdrasil-install-system` and names LykanOS's clean-up. Live medium only. See [INSTALLER.md](INSTALLER.md) |
| `lykanos-security.conf` | `security` | Heimdall in the panel and Settings: its alerts, the "contained" state of Sarina and its release, the files Heimdall holds, and Settings > Wi-Fi & Network > Devices and Wi-Fi checks |
| `lykanos-packages.conf` | `packages` | Agora's own catalog for LykanOS (`/usr/share/lykanos/agora/catalog.json`, edition `lykanos`): what only LykanOS offers, from `[labyrinth]`. See "Agora on LykanOS" below |

## Agora on LykanOS

Agora, the app store, lists what the operating system's `packages` integration names, plus repositories the user adds.
Labyrinth's catalog lives in Labyrinth's image (`/usr/share/labyrinth`), so on LykanOS Agora had no catalog of its own until
2026-11-07. LykanOS's is `profile/airootfs/usr/share/lykanos/agora/catalog.json`; every entry is marked for the edition
`lykanos`, so a copy of the file anywhere else lists nothing.

- **Argus** (decided 2026-11-07): the Argus app and the Argus firmware for the Flipper Zero as one entry and one package,
  `argus-git`. The package holds the app (`qFlipper`, also `argus`; shown as Argus) and the firmware built as an update
  bundle, `/usr/share/argus/firmware/argus-firmware.tgz` (in Argus: Install from file), with a `.dfu` for a Flipper in
  recovery mode. Its recipe is in Labyrinth_OS (`packages/pkgbuilds/argus-git`) and `[labyrinth]` carries it; it is not on
  Labyrinth's ISO and Labyrinth's Agora does not list it.

## The tools in the start menu

Most security tools ship no menu entry, so on a plain desktop they are invisible. `lykanos-tool-menu` (run by a pacman hook
after every install, upgrade or removal, and while the ISO is built) has Sarina's `desktop-audit` write an entry for each
installed command-line tool whose pacman group or description makes it a security tool (`--only-category Security`), in
`/usr/local/share/applications`. Yggdrasil's start menu lists them in its Security folder and starts them in a terminal, with the
shell left open after the tool's help. Only entries marked as Sarina's are replaced or removed. It needs a `sarina-git` with
the `--only-category` flag (Sarina c90f1ae): the one on the 2026-11-08 ISO (`0.1.0.r244`) has it, and the hook ran during
that build. With an older one the hook does nothing. Whether the entries show up in the start menu is not checked yet (that
needs a booted image).

## The look

Deep teal green, decided 2026-11-07 from a reference picture: `#101c1c` at the edges, `#24413c` in the middle, `#437569`
the glow and the accent (white text reads on it; `ANSI_COLOR` in `/etc/os-release` too). Rosewood `#65000b` was added the
same day, BlackArch's kind of red: a glow in the wallpaper, the wolf's eyes, and a logo tone for light backgrounds (on a dark
background it is lifted to `#b0122a`, as Rosewood itself is too dark to read there).

The logo is a wolf's head of maze walls, front on: I made the picture with Gemini from my Labyrinth logo and the first LykanOS logo and
drew more maze walls into it by hand (2026-11-07), so it carries Labyrinth's line work. It replaced a first version drawn the
same day (Labyrinth's arrow turned upside down into a wolf's head), which I did not like as much.

- `assets/logo/lykanos-original.jpg` is the picture as I made it. `scripts/gen-source.py` (Pillow, numpy, potrace,
  rsvg-convert) centres it on a square, traces it as straight segments and straightens every outline (wobbles under 4 px
  become one straight edge, corners stay sharp), so walls drawn with a finger come out straight: `assets/logo/
  lykanos-source.svg` (the outline), `lykanos-source.png` and `lykanos-eyes.png` (the inside of each eye, filled from a point
  in it). `scripts/gen-logo.py` (Pillow) makes the set: white, mint, green, rosewood, grey and black, as transparent PNGs and
  SVGs from that outline (`assets/logo/`, shipped in `/usr/share/lykanos/logo/`), and the duo (mint walls, Rosewood eyes),
  which is the splash logo `lykanos-splash.png` and the icon `lykanos` (`/usr/share/icons/hicolor/*/apps/`;
  `LOGO=lykanos` in os-release). A new picture: replace the original and run both again.
- `scripts/gen-wallpaper.py` (Pillow) makes `/usr/share/wallpapers/lykanos/lykanos.png` (2560x1440): the desktop's
  default wallpaper (`lykanos-branding.conf`) and the login screen's (`/etc/yggdrasil/greeter.conf`).
- Not yet: a boot menu theme and a boot animation (GRUB's plain menu; no Plymouth).

## Heimdall and the firewall

Heimdall (`heimdall-git`, from `[labyrinth]`) is installed but **off**, decided 2026-11-07: before it watches a system full
of security tools it needs a rule for each BlackArch tool, so it neither stops them working nor gets turned against the
system through them. `ufw` and `clamav` come from Arch.

- **How it stays off:** the image has `/etc/heimdall/defaults-off`, so Heimdall's install hook (`heimdall-install.hook`
  -> `heimdall-enable-defaults`, which pacstrap runs while the image is built) switches nothing on. Live session and
  installed system alike. The 2026-11-08 build log shows the hook saying so ("nothing was switched on"). To switch it on:
  `sudo heimdall-enable-defaults --force && sudo systemctl start heimdall`.
- **The rules (2026-11-07):** Heimdall's tool rules (Heimdall `docs/TOOL_RULES.md`, `917d6b0`), in force because the image has
  `/etc/heimdall/security-tools`. For the 29 tools in its catalog (the core list here and a few more), a tool's program file is
  checked against its package: a verified tool's normal work is written as info and a scanner's destinations are summarised;
  a changed tool file is a critical alert; what Sarina starts and an offline tool (hashcat, john) connecting are shown. Not
  covered yet: Wi-Fi monitor mode, payload files, C2 servers. Heimdall stays off until these are tested with the tools in a
  LykanOS VM.
- **During the install** `lykanos-install-cleanup` enables ClamAV's signature updates (`clamav-freshclam.service`).
- **Firewall:** `ufw` is installed but off for now: listeners for the security tools need incoming connections.
  `sudo ufw enable` turns it on. The plan: on, with rules for the tools, for places like a hacker convention (DEF CON),
  where the network around the computer is hostile. (Labyrinth OS has it on, with incoming connections blocked.)
- **Sarina** is part of LykanOS too, and gets access to all the tools once ready (with the rules above).

## Limbo

Limbo (`limbo-git`), the sandbox Labyrinth OS ships for running guest programs in virtual machines, is not on the LykanOS
image and not in LykanOS's Agora catalog. `[labyrinth]` carries it. Not tried on LykanOS.

On a LykanOS host Limbo never gives a guest real hardware: its host policy (Limbo's `internal/hostpolicy`) refuses PCI and
USB passthrough when `/etc/os-release` has `ID=lykanos`, and the old ID `chymaera` as well. That is
hard-coded, so no config file turns it on. Limbo's own plan for LykanOS (its `docs/ARCHITECTURE.md`, "Shipping with
LykanOS") still describes the local package repository and an ISO without Heimdall, from before LykanOS took its packages
from `[labyrinth]`.

## Known limit: locking the live session

The live account has no password. `yggdrasil-idle` only observes unless `mode = act` is set in the user's
`~/.config/yggdrasil/idle.conf`, and the live account has no such file, so the live session never locks by itself. A
manual lock (Super + L) would strand the live user: there is no password to unlock with.

## Not checked yet

The ISO is built (2026-11-08); all of this waits for it to be booted (step 6 of [YGGDRASIL_MOVE.md](YGGDRASIL_MOVE.md)):

- The live medium starts to the Yggdrasil session; signing out shows the login screen.
- Heimdall stays off in the live session and after installing (above; the build log already shows its hook leaving it
  off).
- Settings, Agora, Ghostty, the Task Manager and Saga MD open; `pacman -Syu` reads both Arch and `[labyrinth]`.
- The start menu's Security folder lists the security tools.

## Sarina in the desktop

`sarina-git` from `[labyrinth]` installs an `assistant` integration file, `/usr/share/yggdrasil/integrations/sarina.conf`,
so Yggdrasil shows Sarina in the panel and Settings with the talk key Super + S. The programs that file names
(`/usr/local/bin/labyrinth-sarina-ask`, `-speak`, `-warm`, `-setting`) are part of Labyrinth OS's image, not of the
package, and LykanOS does not ship them. So on LykanOS the talk key has no program behind it, and Settings' Assistant
page hides the controls whose programs are missing (Yggdrasil hides a control whose command is missing). Not fixed yet:
the plan (2026-11-07) moves those programs into a Sarina package, `Sarina/docs/VOICE_PACKAGE_PLAN.md`, tested with me first.
It is on hold until I can test Super + S myself, so it is not built.

## Sarina contracts

None of these exists on LykanOS yet. They are recorded in `CLAUDE.md` as things LykanOS needs from Sarina.

### Crash diagnosis

Omarchy's design is the one to copy: a `systemd-coredump` watcher, a notification carrying the PID, binary and signal,
and a hand-off to the agent with a skill that says how to investigate. In LykanOS the agent is Sarina. The useful
evidence is the coredump, `journalctl`, the program's own logs and GPU driver state. Sarina should propose fixes, not
apply them unattended.

### Sarina as manager of all agents

The plan was to feed Omarchy's agents panel. Omarchy's shell was never on the image, and since 2026-11-07 the desktop is
Yggdrasil, so that panel is not coming. What stands:

- **Built on the Sarina side:** `sarina fleet-usage` (Sarina's usage record), `fleet-status [--suggest]` (every agent's
  records read together, with headroom) and `fleet-launch` (Sarina's `docs/ECOSYSTEM.md` §1).
- **Yggdrasil's way to show something in the top bar** is a `bar-item` integration file (Yggdrasil's
  `docs/INTEGRATIONS.md`); Yggdrasil's own `yggdrasil-agent-usage` add-on, which shows Claude's limits, works that way.
  Sarina's usage shown there would be one more such file. Not built.
- **The launcher stays proposal-first.** It has no way to add an auto-approve flag, needs a human `y` (or `--yes`),
  refuses a prompt that changed after the proposal, and audits every attempt to `~/.local/state/sarina/agent-launch.jsonl`
  for Heimdall (the prompt stored as a hash and a 60-character preview). It is a CLI command only, **not a tool the
  model can call**, and a test enforces that.
- **Other agents' records are untrusted input.** Control and bidi characters are stripped, text truncated, ids and
  numbers validated.

### Proton VPN

**`proton-vpn-gtk-app` is in Arch's official `extra` repo** (4.18.1 when checked), not only the AUR. Proton does not
officially support Arch, but pacman delivers updates the normal signed way, with no packaging on our side. It needs
NetworkManager and a keyring; both are on the image (GNOME Keyring).

- **Install on demand, not baked in.** It needs an account, so it does not serve the offline-core rationale. Sarina
  installs it when asked. Only `wireguard-tools` is on the ISO.
- **Kill switch belongs outside Sarina.** If the VPN must fail closed, that rule lives in nftables (or the app's own
  NetworkManager kill switch), so a crashed or prompt-injected agent cannot leave it open. Sarina may *propose* changes to
  network config; a human approves them, and Heimdall is the natural watcher for that surface.
- **Adapting to updates** means Sarina reading the package's changelog and version delta after `pacman -Syu` and
  flagging what changed for the VPN (NetworkManager connection names, kill-switch behaviour). Known past breakage: Arch
  forum reports of connection errors after specific app versions.
- **Tailscale and Headscale are a different tool, kept as an option.** They build a private mesh between *your own*
  devices over WireGuard; they do not route traffic out through Proton. Useful for connecting a LykanOS box to a team or
  a Flipper rig. Both are in Arch `extra`, so there is nothing to package. Tailscale's coordination server is a hosted
  service; Headscale is the self-hosted replacement for it. Neither is installed or enabled; Sarina installs them on
  request.
- **A VPN that hides the source address can conflict with scoped engagements** and client allow-lists, so it should be a
  toggle, not always on.
