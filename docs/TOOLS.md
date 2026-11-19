# The LykanOS tools and the `[lykanos]` repository

LykanOS's own security tools for **authorized** testing live in a separate, **private** repository, not in this one and
never in Labyrinth OS (Labyrinth's text never mentions
pentest tools). They reach an installed LykanOS system as signed packages through the `[lykanos]` pacman repository.

The first tool is **lykan-webaudit**, an authorized web-application security audit (20 issue classes, live and in source,
each with its fix).

## `[lykanos]`

The release named **packages** of this public repository (`github.com/Artifex-Technologies/LykanOS`) is a pacman
repository: the LykanOS tools, each package and the database signed. An installed system lists it last in
`/etc/pacman.conf` (`profile/airootfs/etc/pacman.conf`):

```
[lykanos]
SigLevel = Required
Server = https://github.com/Artifex-Technologies/LykanOS/releases/download/packages
```

- **Public release, private source.** The source repository is private; the release it publishes to is on this public
  repo, so end users install and update the tools with `pacman -Syu` and no GitHub login.
- **Same key as Labyrinth.** The packages are signed with the Labyrinth key
  (`C9CE784C8BD9B661042075972D3BD4B1E7CD07A5`). A LykanOS system already trusts it: it installs **labyrinth-keyring**
  from `[labyrinth]`, which `pacman-init.service` populates on first boot. Nothing new has to be trusted for
  `[lykanos]`.
- **Last, after `[labyrinth]`.** Like `[labyrinth]`, it can add LykanOS's own software but never replace a base-system
  or desktop package. Its package names are its own (`lykan-webaudit`), so there is no overlap.

## Why it is only in the installed-system `pacman.conf`

The entry is in `profile/airootfs/etc/pacman.conf` (the live and installed system's), **not** in `profile/pacman.conf`
(the one `mkarchiso` uses to build the ISO). `mkarchiso` refreshes every repository in the build config during pacstrap;
a repository whose `packages` release does not exist yet would make that refresh fail and no ISO would build. Until the
owner has run the first publish, `[lykanos]` therefore stays out of the build config. The tools are not in
`profile/packages.x86_64` either — they are installed on demand, not baked into the ISO (see `docs/ARCHITECTURE.md` on
the small tool core).

Once the release exists and carries the tools, adding `lykan-webaudit` to `profile/packages.x86_64` and `[lykanos]` to
`profile/pacman.conf` would put it on the ISO, the same way a `[labyrinth]` package goes on only once Labyrinth's
publish carries it (`CLAUDE.md`, "Know-how"). That is a product decision for the maintainer.

## How a tool gets published

The source repo builds and signs each tool with Labyrinth's key and uploads it to this repo's `packages` release
(`lykanos-publish-repo`, modelled on Labyrinth's publisher, documented in that repository). It runs only on
the PC that holds the signing key. A newer tool reaches LykanOS through `pacman -Syu`, like the desktop.
