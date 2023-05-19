{
  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, crane, fenix, flake-utils, flake-parts, nixpkgs }@inputs: flake-parts.lib.mkFlake { inherit inputs; } ({ moduleWithSystem, ... }: {
    imports = [
      inputs.flake-parts.flakeModules.easyOverlay
    ];

    flake.nixosModules.sable = moduleWithSystem (
      { config }:
      { ... }: {
        imports = [
          ./nix/modules/sable.nix
        ];
      }
    );

    systems = [
      "x86_64-linux"

      # Not actively tested, but may work:
      # "aarch64-linux"
    ];

    perSystem = { config, system, pkgs, ... }:
      let
        pkgs = import nixpkgs {
          system = system;
          overlays = [
            self.overlays.default
          ];
        };
        inherit (pkgs) lib;
      in {
        packages =
          let
            craneLib = crane.lib.${system}.overrideToolchain
              (fenix.packages.${system}.fromToolchainFile {
                file = ./rust-toolchain.toml;
                sha256 = "sha256-kadEI6Hg6v+Xw68334b8XpfNah1pZAJZQ+i6ViN+QyQ=";
              });
            buildSable = features: craneLib.buildPackage {
              src = ./.;
              cargoExtraArgs = "--features \"${lib.concatStringsSep " " features}\"";
            };
          in
          {
            sable = buildSable [ ];
            sable-dev = buildSable [ "sable_ircd/debug" "sable_network/debug" ];
          };

          overlayAttrs = {
            inherit (config.packages) sable sable-dev;
          };

          checks = (import ./nix/tests/sable.nix {
            inherit pkgs;
            sableModule = self.nixosModules.sable;
          });
        };
      });
}
