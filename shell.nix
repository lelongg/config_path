let
  rust-overlay = builtins.fetchTarball {
    url =
      "https://github.com/oxalica/rust-overlay/tarball/2801b291b546dcdcebfa47043fe09f7a0da72880";
    sha256 = "sha256:1mywq4w1z5rhr3k5gavcq01d3m71hwzdc7jc716vxld1n8v2ihzn";
  };
  pkgs = import <nixpkgs> { overlays = [ (import (rust-overlay)) ]; };
in with pkgs;
let
  rustStable =
    rust-bin.stable.latest.minimal.override { extensions = [ "rust-src" ]; };
  rustPlatform = super.makeRustPlatform {
    rustc = rustStable;
    cargo = rustStable;
  };
in mkShell {
  buildInputs = [ clang rustStable openssl pkgconfig ];
  LIBCLANG_PATH = "${llvmPackages.libclang}/lib";
}
