{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    fenix,
    flake-utils,
    nixpkgs,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      toolchain = fenix.packages.${system}.minimal.toolchain;
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs;
          [
            pkg-config
            gcc
            clang
            toolchain
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.buildPlatform.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.CoreFoundation
            pkgs.darwin.apple_sdk.frameworks.CoreServices
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            pkgs.libiconv
          ];
      };

      packages.default = let
      in
        (pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        })
        .buildRustPackage {
          pname = "ned";
          version = "0.0.1";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;

            outputHashes = {
              "nvim-oxi-0.4.2" = "sha256-2OmLhPqFF8KO+vEvpqNWO/ojg1rJR9Alohynk3NGux8=";
            };
          };
        };
    });
}
