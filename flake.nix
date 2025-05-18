{
  description = "Nix Debug Adapter Implementation (DAP)";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    oxalica.url = "github:oxalica/rust-overlay";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-utils,
      oxalica,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import oxalica) ];
        };

        rustToolchain = with pkgs; [
          (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
        ];
      in
      {
        devShell = pkgs.mkShell {
          shellHook = ''
            export CARGO_TARGET_DIR="$(git rev-parse --show-toplevel)/target_dirs/nix_rustc";
          '';
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          buildInputs =
            with pkgs;
            [
              rustToolchain
              pkg-config
              just
              cargo-expand
              # protobuf
              pkg-config
            ]
            ++ pkgs.lib.optionals stdenv.isDarwin [
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.SystemConfiguration
              pkgs.libiconv
            ];
        };
        test = builtins.enable-dap;
      }
    );
}
