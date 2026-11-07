#!/usr/bin/env bash
# Builds the Niflheim OS ISO. Must run on Arch Linux (bare metal, VM, or
# an archlinux Docker container) as root or via sudo — mkarchiso requires it.
#
# What this does, in order:
#   1. Installs BlackArch's signing key + mirrorlist on this build host, via
#      BlackArch's own strap.sh, if not already present.
#   2. Trusts Labyrinth OS's package signing key on this build host, by its
#      full fingerprint: the desktop (Yggdrasil), Sarina, Heimdall and Limbo
#      come from Labyrinth's signed repository ([labyrinth] in profile/pacman.conf).
#   3. Runs mkarchiso against a work copy of profile/, with the service links
#      made there (default target, login screen, network).
#
# Nothing is built locally: since 2026-11-07 every package comes from a
# repository (until then sarina-git was built from a copy of Sarina kept in
# packages/sarina, removed on 2026-10-08).
#
# Output: out/*.iso (or $NIFLHEIM_OUT_DIR/*.iso)
#
# Env: NIFLHEIM_WORK_DIR, NIFLHEIM_OUT_DIR override the default work/ and out/
# locations, for build hosts where those need to sit on a different filesystem.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Overridable because a full BlackArch + desktop pacstrap needs far more room
# than a GitHub runner's root disk has (~14 GB). CI points these at the
# runner's larger scratch disk — see .github/workflows/build-iso.yml.
work_dir="${NIFLHEIM_WORK_DIR:-$repo_root/work}"
out_dir="${NIFLHEIM_OUT_DIR:-$repo_root/out}"
profile_build_dir="$work_dir/profile"

if [[ $EUID -ne 0 ]]; then
  echo "error: run as root (mkarchiso requires it), e.g.: sudo ./scripts/build.sh" >&2
  exit 1
fi

if ! command -v mkarchiso >/dev/null 2>&1; then
  echo "error: mkarchiso not found — install archiso: pacman -S --needed archiso" >&2
  exit 1
fi

if [[ ! -f /etc/pacman.d/blackarch-mirrorlist ]]; then
  echo "==> Installing BlackArch keyring/mirrorlist on build host"
  curl -fsSL https://blackarch.org/strap.sh -o /tmp/blackarch-strap.sh
  bash /tmp/blackarch-strap.sh
else
  echo "==> BlackArch mirrorlist already present, skipping strap.sh"
fi

echo "==> Labyrinth's package signing key on the build host"
# The key is fetched from Labyrinth's repository and trusted only when its fingerprint matches; pacman then checks every
# [labyrinth] package and the database against it (SigLevel = Required).
LABYRINTH_FPR=C9CE784C8BD9B661042075972D3BD4B1E7CD07A5
if ! pacman-key --list-keys "$LABYRINTH_FPR" >/dev/null 2>&1; then
  key_file=$(mktemp)
  curl -fsSL https://raw.githubusercontent.com/Artifex-Technologies/Labyrinth_OS/main/packages/pkgbuilds/labyrinth-keyring/labyrinth.gpg -o "$key_file"
  pacman-key --add "$key_file"
  rm -f "$key_file"
  pacman-key --finger "$LABYRINTH_FPR" 2>/dev/null | tr -d ' ' | grep -q "$LABYRINTH_FPR" \
    || { echo "error: the Labyrinth key fetched does not have fingerprint $LABYRINTH_FPR" >&2; exit 1; }
  pacman-key --lsign-key "$LABYRINTH_FPR"
else
  echo "    already present"
fi

echo "==> Preparing work copy of profile/"
rm -rf "$profile_build_dir"
mkdir -p "$work_dir"
cp -r "$repo_root/profile" "$profile_build_dir"

# Service wiring as symlinks, made here rather than kept in git: mkarchiso copies airootfs over the pacstrapped root as it
# is, and a checkout without symlinks (core.symlinks=false, as this repository's first checkout was) turns a committed link
# into a text file holding its target, which systemd reads as a broken unit (until 2026-11-07 the login screen, the network
# and the default target were such files).
sysd="$profile_build_dir/airootfs/etc/systemd/system"
mkdir -p "$sysd/multi-user.target.wants" "$sysd/bluetooth.target.wants"
ln -sf /usr/lib/systemd/system/graphical.target "$sysd/default.target"
# the login screen: greetd with Yggdrasil's greeter (on the live medium it signs the live user in, greetd.service.d/live.conf)
ln -sf /usr/lib/systemd/system/greetd.service "$sysd/display-manager.service"
ln -sf /usr/lib/systemd/system/NetworkManager.service "$sysd/multi-user.target.wants/NetworkManager.service"
ln -sf /usr/lib/systemd/system/bluetooth.service "$sysd/bluetooth.target.wants/bluetooth.service"
ln -sf /etc/systemd/system/pacman-init.service "$sysd/multi-user.target.wants/pacman-init.service"
ln -sf /dev/null "$sysd/systemd-firstboot.service"

echo "==> Running mkarchiso"
mkdir -p "$out_dir"
mkarchiso -v -w "$work_dir/iso" -o "$out_dir" "$profile_build_dir"

echo "==> Done: $out_dir"
