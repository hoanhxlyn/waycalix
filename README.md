# waycalix

Nix flake for [waycal](https://github.com/ForrestKnight/waycal) — a tiny Wayland calendar popup.

Packages waycal with a home-manager module and binary cache via [Cachix](https://app.cachix.org/cache/waycalix).

## Usage

Add to your flake inputs:

```nix
waycalix.url = "github:hoanhxlyn/waycalix";
```

### Home-manager module

```nix
{
  imports = [inputs.waycalix.homeManagerModules.default];

  programs.waycal = {
    enable = true;
    css = ''
      /* optional custom styles */
    '';
  };
}
```

### Cachix binary cache

Run once to avoid building from source:

```sh
cachix use waycalix
```

Or add to your nix settings:

```nix
nix.settings = {
  extra-substituters = ["https://waycalix.cachix.org"];
  extra-trusted-public-keys = [
    "waycalix.cachix.org-1:nGZcdc7jmLn1cxr7t2mVOpZ+xpChExJWk9igdKX4yg0="
  ];
};
```

## License

MIT.
