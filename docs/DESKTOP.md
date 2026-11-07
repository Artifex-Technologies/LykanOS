# Desktop: Yggdrasil

Niflheim's desktop is [Yggdrasil](https://github.com/Artifex-Technologies/Yggdrasil), the desktop environment written in
Rust that Labyrinth OS uses, running on Hyprland. I decided on 2026-11-07 to use the same desktop as Labyrinth OS rather
than keep one of Niflheim's own; the move is recorded in [YGGDRASIL_MOVE.md](YGGDRASIL_MOVE.md). Until 2026-11-07 the
desktop was KDE Plasma (SDDM, the MacTahoe theme, Kvantum) with an Omarchy-style Hyprland session and two Waybar looks
beside it. Those files are gone from the profile.

This page says what Niflheim takes from Yggdrasil, where the packages come from and what Niflheim adds. For the desktop
itself, Yggdrasil's own documentation is the reference: its README (the programs and the keyboard shortcuts),
`docs/INTEGRATIONS.md` and `docs/INSTALLER.md`.

## Status

| Piece | State |
| --- | --- |
| Package list | Resolves against Arch, BlackArch and `[labyrinth]` (checked 2026-11-07, when the move was made) |
| ISO with Yggdrasil | CI could not reach `mkarchiso` until 2026-11-07 (see the README's Status); the first run with the fixes started that day |
| Live session, login screen, Settings, Agora, installer | **Not booted in a VM yet** (step 6 of [YGGDRASIL_MOVE.md](YGGDRASIL_MOVE.md)) |
| The desktop packages themselves | Run on Labyrinth OS, which installs the same ones; not yet with Niflheim |
| Real GPU hardware | Not tried with Niflheim |
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
  `yggdrasil-colorpicker`, `yggdrasil-calculator`, `yggdrasil-spell` (with `hunspell-en_us`), `yggdrasil-installer`,
  `yggdrasil-theme` (GTK and Qt look, the pointer) and `liki-icon-theme`.
- **Terminal:** Ghostty (Super + Return).
- **Browser:** Firefox.
- **Saved passwords:** GNOME Keyring (`gnome-keyring`), the same choice Yggdrasil made; I kept it on 2026-11-07.
- **Wi-Fi, Bluetooth and sound:** Yggdrasil's Settings, over NetworkManager, BlueZ (`bluez`, `bluez-utils`) and PipeWire.
- **Small tools the session uses:** `grim`, `slurp`, `wl-clipboard`, `brightnessctl`, `playerctl`, `libnotify`,
  `noto-fonts-emoji`, and `btop` (Yggdrasil's Ctrl + Shift + Esc opens it in a terminal).

Yggdrasil's other programs (the video player, the weather and the store's apps) are not on the image; Agora installs them.

## Login screen

greetd runs Yggdrasil's greeter inside `cage`, a small compositor, on a console of its own (VT 7):
`profile/airootfs/etc/greetd/config.toml`. The greeter account (`greeter`) comes from the greetd package.

On the live medium a drop-in, `etc/systemd/system/greetd.service.d/live.conf`, makes greetd read `etc/greetd/live.toml`
instead. That file is the same login screen plus an `initial_session`: once per start-up the live user is signed in to
`yggdrasil-session` without a password. After signing out, the normal login screen comes. `niflheim-install-cleanup`
deletes both files, so an installed system starts at the login screen.

`display-manager.service` (pointing at `greetd.service`) is one of the links `scripts/build.sh` makes in the work copy of
the profile; see CLAUDE.md for why they are not in git.

## Where the packages come from

| Packages | Repository |
| --- | --- |
| `yggdrasil-*`, `liki-icon-theme`, `heimdall-git`, `sarina-git`, `labyrinth-keyring` | `[labyrinth]`: Labyrinth OS's signed package repository |
| `hyprland`, `greetd`, `cage`, `ghostty`, `gnome-keyring`, `btop`, `ufw`, `clamav` and the rest | Arch (`core`, `extra`, `multilib`) |
| the security tools | Arch and `[blackarch]` (build only; see [ARCHITECTURE.md](ARCHITECTURE.md)) |

Nothing is built in this repository. Until 2026-11-07 `sarina-git` was built into a local `[niflheim]` repository from
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
- **On the installed system**, `niflheim-install-cleanup` does the same during the install; `pacman-init.service` stays as
  a safety net and runs only when there is no keyring.

So `pacman -Syu` brings desktop, Heimdall and Sarina updates the same day Labyrinth gets them.

## What Niflheim adds: integration files

Yggdrasil never names an operating system; it reads integration files (Yggdrasil's `docs/INTEGRATIONS.md`). Niflheim's
are in `profile/airootfs/usr/share/yggdrasil/integrations/`:

| File | Kind | What it does |
| --- | --- | --- |
| `niflheim-branding.conf` | `branding` | The accent colour `#437569`, the wallpaper and the splash logo (see "The look" below). The name and logo come from `/etc/os-release` |
| `niflheim-installer.conf` | `installer` | Points the installer window at `yggdrasil-install-system` and names Niflheim's clean-up. Live medium only. See [INSTALLER.md](INSTALLER.md) |
| `niflheim-security.conf` | `security` | Heimdall in the panel and Settings: its alerts, the "contained" state of Sarina and its release, the files Heimdall holds, and Settings > Wi-Fi & Network > Devices and Wi-Fi checks |

## The look

Deep teal green, decided 2026-11-07 from a reference picture: `#101c1c` at the edges, `#24413c` in the middle, `#437569`
the glow and the accent (white text reads on it; `ANSI_COLOR` in `/etc/os-release` too). The logo is a snake wound round a
labyrinth "A", drawn to echo the Arch logo; a draft, made on 2026-11-07, to replace freely.

- `assets/logo/niflheim-source.png` is the drawing (white lines on black). `scripts/gen-logo.py` (Pillow, potrace) makes
  the set from it: white, mint, green, grey and black, as transparent PNGs and SVGs (`assets/logo/`, shipped in
  `/usr/share/niflheim/logo/`), the splash logo `niflheim-splash.png`, and the icon `niflheimos` (mint, in
  `/usr/share/icons/hicolor/*/apps/`; `LOGO=niflheimos` in os-release). Run it again after changing the drawing.
- `scripts/gen-wallpaper.py` (Pillow) makes `/usr/share/wallpapers/niflheim/niflheim-green.png` (2560x1440): the desktop's
  default wallpaper (`niflheim-branding.conf`) and the login screen's (`/etc/yggdrasil/greeter.conf`).
- Not yet: a boot menu theme and a boot animation (GRUB's plain menu; no Plymouth).

## Heimdall and the firewall

Heimdall (`heimdall-git`, from `[labyrinth]`) is installed but **off**, decided 2026-11-07: before it watches a system full
of security tools it needs a rule for each BlackArch tool, so it neither stops them working nor gets turned against the
system through them. `ufw` and `clamav` come from Arch.

- **How it stays off:** the image has `/etc/heimdall/defaults-off`, so Heimdall's install hook (`heimdall-install.hook`
  -> `heimdall-enable-defaults`, which pacstrap runs while the image is built) switches nothing on. Live session and
  installed system alike. To switch it on: `sudo heimdall-enable-defaults --force && sudo systemctl start heimdall`.
- **During the install** `niflheim-install-cleanup` enables ClamAV's signature updates (`clamav-freshclam.service`).
- **Firewall:** `ufw` is installed but off for now: listeners for the security tools need incoming connections.
  `sudo ufw enable` turns it on. The plan: on, with rules for the tools, for places like a hacker convention (DEF CON),
  where the network around the computer is hostile. (Labyrinth OS has it on, with incoming connections blocked.)
- **Sarina** is part of Niflheim too: once she is ready, she gets access to all the tools (with the rules above).

## Known limit: locking the live session

The live account has no password. `yggdrasil-idle` only observes unless `mode = act` is set in the user's
`~/.config/yggdrasil/idle.conf`, and the live account has no such file, so the live session never locks by itself. A
manual lock (Super + L) would strand the live user: there is no password to unlock with.

## Not checked yet

All of this waits for the first ISO (step 6 of [YGGDRASIL_MOVE.md](YGGDRASIL_MOVE.md)):

- The live medium starts to the Yggdrasil session; signing out shows the login screen.
- Heimdall stays off in the live session and after installing (above).
- Settings, Agora and Ghostty open; `pacman -Syu` reads both Arch and `[labyrinth]`.

## Sarina in the desktop

`sarina-git` from `[labyrinth]` installs an `assistant` integration file, `/usr/share/yggdrasil/integrations/sarina.conf`,
so Yggdrasil shows Sarina in the panel and Settings with the talk key Super + S. The programs that file names
(`/usr/local/bin/labyrinth-sarina-ask`, `-speak`, `-warm`, `-setting`) are part of Labyrinth OS's image, not of the
package, and Niflheim does not ship them. So on Niflheim the talk key has no program behind it, and Settings' Assistant
page hides the controls whose programs are missing (Yggdrasil hides a control whose command is missing). Not fixed yet:
the plan (2026-11-07) moves those programs into a Sarina package, `Sarina/docs/VOICE_PACKAGE_PLAN.md`, tested with me first.

## Sarina contracts

None of these exists yet. They are recorded in `CLAUDE.md` as things Niflheim needs from Sarina.

### Crash diagnosis

Omarchy's design is the one to copy: a `systemd-coredump` watcher, a notification carrying the PID, binary and signal,
and a hand-off to the agent with a skill that says how to investigate. In Niflheim the agent is Sarina. The useful
evidence is the coredump, `journalctl`, the program's own logs and GPU driver state. Sarina should propose fixes, not
apply them unattended.

### Sarina as manager of all agents

The plan was to feed Omarchy's agents panel. Omarchy's shell was never on the image, and since 2026-11-07 the desktop is
Yggdrasil, so that panel is not coming. What stands:

- **Built on the Sarina side:** `sarina fleet-usage` (Sarina's usage record), `fleet-status [--suggest]` (every agent's
  records read together, with headroom) and `fleet-launch` (Sarina's `docs/ECOSYSTEM.md` §1).
- **Yggdrasil's way to show something in the top bar** is a `bar-item` integration file (Yggdrasil's
  `docs/INTEGRATIONS.md`); Yggdrasil's own `yggdrasil-agent-usage` add-on, which shows Claude's limits, works that way.
  Sarina showing her usage there would be one more such file. Not built.
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
  devices over WireGuard; they do not route traffic out through Proton. Useful for connecting a Niflheim box to a team or
  a Flipper rig. Both are in Arch `extra`, so there is nothing to package. Tailscale's coordination server is a hosted
  service; Headscale is the self-hosted replacement for it. Neither is installed or enabled; Sarina installs them on
  request.
- **A VPN that hides the source address can conflict with scoped engagements** and client allow-lists, so it should be a
  toggle, not always on.
