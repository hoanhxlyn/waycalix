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
    flake-utils.lib.eachDefaultSystem (system: let
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
        packages = [rustToolchain];
      };
    });
}
