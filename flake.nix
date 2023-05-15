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
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, crane, fenix, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system: {
      packages.default =
        let
          craneLib = crane.lib.${system}.overrideToolchain
            fenix.packages.${system}.minimal.toolchain;
        in
        craneLib.buildPackage {
          src = ./sable_ircd;
        };
    });
}
