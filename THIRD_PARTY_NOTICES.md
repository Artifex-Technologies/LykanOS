# Third-party notices

Files in this repository that are adapted from other projects, and their licences.

Nothing in the tree today is adapted from another project's code under terms that need a notice here. `profile/` is a
fork of archiso's `releng` profile (archiso is GPL-3.0-or-later; this repository is GPL-3.0): `profiledef.sh`,
`packages.x86_64`, `pacman.conf` and the mkinitcpio files say so in their headers, and `syslinux/` and `efiboot/` are
releng's boot menus with Niflheim's names. The desktop (Yggdrasil), Heimdall and Sarina are installed as packages, each with its own
licence file.

Until 2026-11-07 parts of the Hyprland session configuration and the `niflheim-desktop-mode` script were adapted from
[Omarchy](https://github.com/omacom/omarchy) (MIT, Copyright (c) David Heinemeier Hansson, tag `v4.0.4`). They were
removed when the desktop moved to Yggdrasil (commit 3193bca) and remain only in the git history before it.
