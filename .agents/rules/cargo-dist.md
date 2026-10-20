# Cargo-Dist & CI Workflow Rules

1. **Cargo-Dist Integrity**:
   - `.github/workflows/release.yml` is generated and checked by `cargo-dist`.
   - Never manually strip comments or modify `release.yml` without ensuring `allow-dirty = ["ci"]` is configured in `dist-workspace.toml`.
   - Always run `dist plan` locally to verify workflow integrity before finalizing changes to release configurations.
   - If `release.yml` is out of sync with `dist-workspace.toml`, regenerate it using `dist generate-ci`.

2. **Clean Configuration Preference**:
   - The user prefers clean configuration files without boilerplate comments where possible. When stripping comments, always verify against tool integrity checks (like `cargo-dist`).

3. **Release-Plz & Cargo-Dist Integration**:
   - `release-plz.toml` has `git_release_enable = false`.
   - `release-plz` creates version bump PRs, updates changelogs, pushes git tags, and publishes to crates.io.
   - `cargo-dist` builds the binaries and creates the GitHub Release with attached assets and Homebrew formula.

4. **Packaging & Releases**:
   - `CHOCO_API_KEY` and `HOMEBREW_TAP_TOKEN` are GitHub repository secrets inside `easy-docker`.
   - Chocolatey packages are defined in `packaging/chocolatey/` and tested via `.github/workflows/chocolatey.yml`.
