{ pkgs ? import <nixpkgs> {
    overlays = [
      (import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz))
    ];
  }
}:
pkgs.mkShell {
  buildInputs = with pkgs; [
    #latest.rustChannels.stable.rust
    latest.rustChannels.nightly.rust
    git
    openssl
    pkgconfig
    jq
    gptfdisk
    carnix
  ];

  RUST_BACKTRACE = "1";
  NIX_PATH = "nixpkgs=${pkgs.path}";
}
