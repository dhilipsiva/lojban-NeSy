# Changelog

All notable changes to the nibli workspace are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the workspace adheres to lockstep [Semantic Versioning](https://semver.org/)
for its published crates (Tier A in
[DOCS_TODO.md](DOCS_TODO.md)'s decisions table). The WIT component ABI version
(`nibli:engine@…` in `wit/world.wit`) is **independent** of crate semver.

During 0.x, minor versions may break APIs; every release documents its changes
here first.

## [Unreleased]

No release has been tagged yet — this section accumulates everything that will
ship in the first tagged release (see DOCS_TODO's release track: R0 packaging
landed; R1 first GitHub Release and R2 first crates.io publish are open).

### Added

- Workspace release packaging (R0): lockstep `[workspace.package]` version
  `0.1.0` with shared license/repository/homepage inherited by every member,
  `[workspace.dependencies]` (path + version) for the internal crates
  (internal *dev*-dependencies stay path-only on purpose — they are stripped
  at publish and must not constrain the publish order), `publish = false` on
  the non-publishable tier, per-crate descriptions, this CHANGELOG, and the
  `just release-check` consistency gate. No public release and no crates.io
  publish happened in R0.

[Unreleased]: https://github.com/dhilipsiva/nibli/commits/main
