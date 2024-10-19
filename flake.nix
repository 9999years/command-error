{
  description = "Detailed error messages and status checking for `std::process::Command`";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    systems.url = "github:nix-systems/default";
    crane.url = "github:ipetkov/crane";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  nixConfig = {
    extra-substituters = ["https://cache.garnix.io"];
    extra-trusted-substituters = ["https://cache.garnix.io"];
    extra-trusted-public-keys = ["cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="];
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    systems,
    crane,
    advisory-db,
    rust-overlay,
  }: let
    eachSystem = nixpkgs.lib.genAttrs (import systems);
  in {
    _pkgs = eachSystem (
      localSystem:
        import nixpkgs {
          inherit localSystem;
          overlays = [
            rust-overlay.overlays.default

            (final: prev: {
              rustToolchain = final.pkgsBuildHost.rust-bin.stable.latest.default.override {
                extensions = ["llvm-tools-preview"];
              };

              craneLib = (crane.mkLib final).overrideToolchain final.rustToolchain;
            })
          ];
        }
    );

    packages = eachSystem (
      system: let
        pkgs = self._pkgs.${system};
        inherit (pkgs) lib;
        packages = pkgs.callPackage ./nix/makePackages.nix {inherit inputs;};
      in
        (lib.filterAttrs (name: value: lib.isDerivation value) packages)
        // {
          default = packages.command-error;
          docs = packages.command-error-docs;
          docs-tarball = packages.command-error-docs-tarball;

          # This lets us use `nix run .#cargo` to run Cargo commands without
          # loading the entire `nix develop` shell (which includes
          # `rust-analyzer`).
          #
          # Used in `.github/workflows/release.yaml`.
          cargo = pkgs.cargo;
        }
    );

    checks = eachSystem (system: self.packages.${system}.command-error.checks);

    devShells = eachSystem (system: {
      default = self.packages.${system}.command-error.devShell;
    });
  };
}
