# waycalix

Nix flake wrapping [waycal](https://github.com/ForrestKnight/waycal) (Rust/GTK4 Wayland calendar popup).
This repo owns the packaging and home-manager module, not the Rust source.

## Structure

- `flake.nix` — package definition + `homeManagerModules.default`
- `src/` — upstream Rust source (edit only to patch, not to add features)
- `.github/workflows/cachix.yml` — pushes build artifacts to cachix `waycalix` on push to `main`
- `.github/workflows/update.yml` — auto-syncs upstream waycal + flake inputs (1st/15th of month)

## Commands

```sh
nix build              # build the package
nix develop            # enter dev shell (rust toolchain + cachix)
nix flake check        # validate flake outputs
```

No cargo-based workflow — always build through nix.

## Home-manager module

Options under `programs.waycal`:
- `enable` — bool
- `package` — override the default package
- `css` — `nullOr lines`, written to `~/.config/waycal/style.css`

CSS loading order in the app: built-in defaults → layout → user file (last wins). The user file can override `@define-color` values and `.waycal-root` font rules.

## Cachix

Public key: `waycalix.cachix.org-1:nGZcdc7jmLn1cxr7t2mVOpZ+xpChExJWk9igdKX4yg0=`

CI requires `CACHIX_AUTH_TOKEN` repo secret.

## Version sync

`version` in `flake.nix` (line ~57) must match `version` in `Cargo.toml`. Update both together.

## Auto-update

`.github/workflows/update.yml` runs on the 1st and 15th of each month:
1. Updates nix flake inputs
2. Pulls latest `src/`, `Cargo.toml`, `Cargo.lock` from upstream ForrestKnight/waycal
3. Syncs version from `Cargo.toml` into `flake.nix`
4. Builds to verify, then commits and pushes if anything changed

Can also be triggered manually via `workflow_dispatch`.
