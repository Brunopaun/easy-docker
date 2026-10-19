# Cargo-Dist & CI Workflow Rules

1. **Cargo-Dist Integrity**:
   - `.github/workflows/release.yml` is generated and checked by `cargo-dist`.
   - Never manually strip comments or modify `release.yml` without ensuring `allow-dirty = ["ci"]` is configured in `dist-workspace.toml`.
   - Always run `dist plan` locally to verify workflow integrity before finalizing changes to release configurations.
   - If `release.yml` is out of sync with `dist-workspace.toml`, regenerate it using `dist generate-ci`.

2. **Clean Configuration Preference**:
   - The user prefers clean configuration files without boilerplate comments where possible. When stripping comments, always verify against tool integrity checks (like `cargo-dist`).

3. **Packaging & Releases**:
   - `CHOCO_API_KEY` and `HOMEBREW_TAP_TOKEN` are GitHub repository secrets inside `easy-docker`.
   - Chocolatey packages are defined in `packaging/chocolatey/` and tested via `.github/workflows/chocolatey.yml`.
