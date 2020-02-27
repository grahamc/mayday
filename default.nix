{ pkgs ? import <nixpkgs> {} }:
let
  exactSource = import ./exact-source.nix;

  cargo_nix = pkgs.callPackage ./Cargo.nix {
    defaultCrateOverrides = pkgs.defaultCrateOverrides // {
      mayday = { ... }: {
        src = exactSource ./. [
          ./src/cli.rs
          ./src/logging.rs
          ./src/main.rs
          ./src/servers.rs
          ./src/tailer.rs
          ./src/vault.rs
        ];
      };
    };
  };
in cargo_nix.rootCrate.build
