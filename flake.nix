{
  inputs = {
    nixpkgs.url = "nixpkgs";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-parts,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

      perSystem = {
        lib,
        pkgs,
        self',
        ...
      }: {
        packages.default = pkgs.callPackage ./derivation.nix {inherit self;};

        devShells.default = with pkgs;
          mkShell {
            inputsFrom = [self'.packages.default];

            buildInputs = [
              clippy
              rustfmt
              rust-analyzer
            ];

            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };

        formatter = pkgs.alejandra;
      };
    };
}
