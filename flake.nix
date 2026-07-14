{
  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.xz";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    {
      homeManagerModules.default = {
        config,
        lib,
        pkgs,
        ...
      }: let
        cfg = config.programs.waycal;
      in {
        options.programs.waycal = {
          enable = lib.mkEnableOption "waycal, a tiny Wayland calendar popup";
          package = lib.mkOption {
            type = lib.types.package;
            default = self.packages.${pkgs.stdenv.hostPlatform.system}.waycal;
            defaultText = lib.literalExpression "inputs.waycal.packages.<system>.waycal";
            description = "The waycal package to use.";
          };
          css = lib.mkOption {
            type = lib.types.nullOr lib.types.lines;
            default = null;
            description = "CSS stylesheet written to ~/.config/waycal/style.css.";
          };
        };
        config = lib.mkIf cfg.enable {
          home.packages = [cfg.package];
          xdg.configFile."waycal/style.css" = lib.mkIf (cfg.css != null) {
            text = cfg.css;
          };
        };
      };
    }
    // flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };
      rustToolchain = pkgs.rust-bin.stable.latest.default;
    in {
      packages.waycal = pkgs.rustPlatform.buildRustPackage {
        pname = "waycal";
        version = "0.2.0";

        src = ./.;

        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.wrapGAppsHook4
          rustToolchain
        ];

        buildInputs = [
          pkgs.gtk4
          pkgs.gtk4-layer-shell
        ];

        meta = {
          description = "Tiny Wayland calendar popup: keyboard day navigation, Enter copies the date (dd/mm/yyyy)";
          homepage = "https://github.com/ForrestKnight/waycal";
          license = pkgs.lib.licenses.mit;
          mainProgram = "waycal";
          platforms = pkgs.lib.platforms.linux;
        };
      };

      packages.default = self.packages.${system}.waycal;

      devShells.default = pkgs.mkShell {
        inputsFrom = [self.packages.${system}.waycal];
        packages = [rustToolchain pkgs.cachix];
      };
    });
}
