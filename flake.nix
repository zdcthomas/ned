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

      toolchain = fenix.packages.${system}.complete.toolchain;

      ned =
        (pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        })
        .buildRustPackage {
          pname = "ned";
          version = "0.0.1";
          nativeBuildInputs = with pkgs; [
            pkg-config
            luajit
            gcc
            gnumake
            toolchain
          ];

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs;
          [
            (
              writers.writeBashBin "b" ''
                rm -rf ./lua
                mkdir -p lua
                cargo build
                cp ./target/debug/libned.so ./lua/ned.so
              ''
            )
            (
              writers.writeBashBin "b-old" ''
                rm -rf ./lua
                mkdir -p lua
                cp ${ned}/lib/libned.so ./lua/ned.so
              ''
            )
            pkg-config
            luajit
            gnumake
            gcc
            # clang
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

      packages.default = pkgs.stdenv.mkDerivation {
        src = ./.;
        name = "install";
        installPhase = ''
          mkdir -p $out/lua
          cp -r ${ned}/lib/libned.so $out/lua/ned.so

        '';
      };
    });
}
