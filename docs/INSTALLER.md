# Installer

How LykanOS gets from the live USB onto a disk, why it is built that way, and what has and has not been checked.

Since 2026-11-07 LykanOS installs with Yggdrasil's installer: the window `yggdrasil-installer` and its install program
`yggdrasil-install-system`, both in the `yggdrasil-installer` package from `[labyrinth]`. It is the same program Labyrinth
OS installs with. I moved it from Labyrinth to Yggdrasil on 2026-11-07 (Yggdrasil 0c54aa0, Labyrinth 2b18499) so both
systems share one installer instead of keeping copies. What is LykanOS's own is one integration file and one clean-up
script. Until 2026-11-07 the installer was Calamares (a vendored PKGBUILD and a config that had never been run); it is
removed.

## Status

| Piece | State |
| --- | --- |
| `lykanos-installer.conf` | Written 2026-11-07 |
| `yggdrasil-install-system` | Tested as Labyrinth's program in QEMU/OVMF VMs (2026-10-23 and 2026-10-25; Yggdrasil's `docs/INSTALLER.md`). **Never run on LykanOS** |
| `yggdrasil-installer` in `[labyrinth]` | The build published on 2026-11-06 (`0.1.0.r420`) has only the window, not `yggdrasil-install-system`. Yggdrasil 0c54aa0 raised the package's version; Labyrinth's daily publish on 2026-11-08 puts the build with the program into `[labyrinth]` |
| `lykanos-install-cleanup` | Updated 2026-11-07 (greetd, the Labyrinth key, Heimdall). An earlier version was simulated against a mock target root. **Never run in a real install** |
| zstd squashfs | Set in `profiledef.sh`. **ISO size and install time not measured** |

## Starting it

- **Launchers:** "Install LykanOS" in the start menu (`profile/airootfs/usr/share/applications/lykanos-install.desktop`,
  in the System folder) and the same launcher in the live user's Desktop folder (`home/live/Desktop/install-lykanos.desktop`).
  Both run `yggdrasil-installer`.
- **UEFI only.** The install program stops on a computer started without UEFI ("installs on UEFI computers only"). The
  medium itself also starts on BIOS computers (`bootmodes` in `profiledef.sh` has `bios.syslinux` as well as
  `uefi.systemd-boot`), so the live session works there, but the graphical installer does not.
- **`archinstall`** (from releng's package list) stays as a command-line, scripted alternative. It installs from the
  network, not from the medium.

## Design: copy the live image, do not rebuild it

```
mkarchiso  ──►  airootfs.sfs  (zstd squashfs, inside the ISO)
                    │
   live boot  ──►  /run/archiso/bootmnt/lykanos/x86_64/airootfs.sfs
                    │
   yggdrasil-installer       language, keyboard, time zone, disk, account, summary
     │  pkexec yggdrasil-install-system   (the choices as JSON on standard input)
     ▼
   checks          UEFI, the disk not in use, at least 30 GB
   disk            GPT: a 1 GiB EFI partition and the rest (or partitions chosen by hand)
                   optional LUKS2; btrfs with @, @home, @cache, @log
   copy            unsquashfs the medium's image onto the disk
   inside          lykanos-install-cleanup, then language, keyboard, time zone, the
                   computer's name, the account (wheel, sudo; root locked), greetd
   boot            start-up image (mkinitcpio), GRUB (and the firmware's fallback path)
```

The installed system is a copy of what the user just ran, not a fresh `pacstrap`. That is why the install is fast and
why it works with no network: nothing is resolved, downloaded or built on the target. It also matches the project's rule
that the live session must be useful air-gapped; the installer inherits the same property. The cost is that the
installed system starts as a snapshot of the ISO's package versions, so the first thing a connected user should do is
`pacman -Syu`.

The program finds the image at `/run/archiso/bootmnt/<ID>/x86_64/airootfs.sfs`, where `ID` is `lykanos` from
`/etc/os-release`. That has to equal `install_dir` in `profiledef.sh`: change one and you must change the other (or set
`medium` in `lykanos-installer.conf`).

### What `lykanos-installer.conf` sets

`profile/airootfs/usr/share/yggdrasil/integrations/lykanos-installer.conf`:

| Key | Value |
| --- | --- |
| `command` | `/usr/bin/yggdrasil-install-system` |
| `name` | `LykanOS` (messages, the boot menu, the EFI entry, the disk labels) |
| `cleanup` | `/usr/local/bin/lykanos-install-cleanup` |
| `reserved_logins` | `heimdall sarina` (beyond every account the medium already has, root, live and greeter) |
| `groups` | users, lp, video, network, storage, wheel, audio, kvm and `wireshark` (capturing without root); only those the system has |

Everything else is the program's default (Yggdrasil's `docs/INSTALLER.md` lists the keys): kernel options `quiet`; a
disk of at least 30 GB; no `finish` program;
the plain GRUB menu (no `boot_menu_theme`); and no Browsers or optional-software pages. The file is live-medium only:
the clean-up deletes it, with the launcher.

### Why speed comes from where it does

Install time is dominated by two things, and both are addressed:

1. **Unpacking the image.** The program runs `unsquashfs`, so the bound is decompression plus disk writes.
   `profile/profiledef.sh` builds the image with zstd instead of releng's xz. zstd decompresses several times faster;
   the price is a larger ISO.
2. **Rebuilding the initramfs.** The live image carries archiso's initramfs config, which compresses with `xz -9e`.
   `lykanos-install-cleanup` removes it, so the installed system uses Arch's stock config: zstd, and a single `default`
   image with no fallback.

**Neither effect has been measured.** The direction is well understood, the magnitude is not, and the ISO size increase
is a real trade-off. Measure both from a built ISO before relying on this, and revert `airootfs_image_tool_options` to
releng's xz line if the size is not acceptable. The original line is kept in a comment in `profiledef.sh`.

### What `lykanos-install-cleanup` does, and why each step exists

`profile/airootfs/usr/local/bin/lykanos-install-cleanup` runs inside the new system right after the copy, before the
program sets up the account and builds the start-up image. Every step answers something specific about the live image
not being an installable root:

| Step | Why |
| --- | --- |
| Copy `/home/live/.config` into `/etc/skel`, if there is one (owned by root, readable by all) | Desktop config for new accounts. Since 2026-11-07 the image has no `.config` for the live user (Yggdrasil writes its own at the first sign-in), so this does nothing today |
| Delete the live sudoers file, the polkit rule, greetd's `live.toml` and its `greetd.service.d/live.conf` drop-in | They exist so a passwordless account can do anything and is signed in by itself; on an installed system they would grant that to every `wheel` user and sign in a user that no longer exists. The install program writes the installed system's own sudoers file |
| Delete `sysusers.d/live.conf`, `tmpfiles.d/live-home.conf` and `/home/live` | They recreate the live user on every boot |
| Delete the installer launcher and `lykanos-installer.conf` | An installed system has no live image to copy, so the installer has nothing to offer there |
| Delete `/etc/mkinitcpio.conf.d/archiso.conf` | Its `HOOKS` would override the stock config, and its `xz -9e` slows the initramfs build |
| Regenerate `/etc/mkinitcpio.d/linux.preset` from `/usr/share/mkinitcpio/hook.preset` | The live preset is archiso's and builds an initramfs that looks for the live squashfs. Arch generates the preset from this template, so we use the same template rather than a hand-written copy |
| Restore `/boot/vmlinuz-linux` | mkarchiso runs `find /boot -mindepth 1 -delete` on the image, so the target has no kernel. The package's copy under `/usr/lib/modules/*/vmlinuz` survives, and the script installs it the way Arch's own pacman hook does |
| Initialise the pacman keyring if there is none, then put in the archlinux, blackarch and labyrinth keys | `pacman -Syu` needs the Labyrinth key (the desktop comes from the signed `[labyrinth]`), and Sarina needs a keyring to install anything later. mkarchiso builds the image without a keyring (`pacstrap -G`), and the live one made at start-up is not in the image, so the script normally creates it |
| Enable `clamav-freshclam.service` | ClamAV's signature updates, for the installed system only. Heimdall stays off (`/etc/heimdall/defaults-off`, kept on the installed system) and so does the firewall (`ufw`); see [DESKTOP.md](DESKTOP.md) |

It fails loudly (non-zero exit) if the kernel is missing or the preset template is empty or malformed. Both would
otherwise produce an install that completes and then does not boot.

When you add a live-only file to `profile/airootfs/`, add it to the `live_only` list in that script too. Nothing checks
the two lists against each other.

### Offline

The installer has no network check, and LykanOS's integration file names no Browsers or optional-software lists, so
nothing is fetched during the install. The copy, the clean-up and the initramfs rebuild need no network.

## Not done

- **Never run on LykanOS.** The first real test (step 6 of [YGGDRASIL_MOVE.md](YGGDRASIL_MOVE.md)) is to boot the built
  ISO in a UEFI VM, install, reboot and sign in. Things most likely to need a fix: the image path, the keyring step and
  the clean-up's list of live files.
- **The `yggdrasil-installer` package.** The ISO needs one from `[labyrinth]` that includes `yggdrasil-install-system`
  (see Status): an ISO built before Labyrinth's publish on 2026-11-08 has an installer window with no program to run.
- **BIOS computers.** Only `archinstall` installs there.
- **Branding.** The name and logo are LykanOS's; the boot menu is GRUB's plain one (no theme yet).
- **Required disk space.** The program's default, 30 GB. The real installed size has not been measured.
- **Encryption and keyboard layouts.** GRUB reads the encryption password with a US layout (a known limit of the
  program; Yggdrasil's `docs/INSTALLER.md`).
- **Agent-assisted install.** Install-time tool selection is Sarina's job and does not exist yet; see
  [ARCHITECTURE.md](ARCHITECTURE.md).
