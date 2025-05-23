{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      rustScanning = pkgs.rustPlatform.buildRustPackage {
        pname = "rust-scanning";
        version = "0.1.0";
        src = self;
        cargoLock.lockFile = ./Cargo.lock;
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = [ toolchain ];
      };

      packages.${system}.default = rustScanning;
    };
}
