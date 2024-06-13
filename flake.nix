{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      # inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
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
        .buildRustPackage ({
            pname = "ned";
            version = "0.0.1";

            nativeBuildInputs = with pkgs; [
              pkg-config
              lua
              luajit
            ];
            buildInputs = with pkgs; [
              pkg-config
              luajit
              lua
              gcc
              gnumake
              toolchain
            ];

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };
          }
          // pkgs.lib.attrsets.optionalAttrs pkgs.stdenv.isDarwin {
            RUSTFLAGS = [
              "-C link-arg=-undefined"
              "-C link-arg=dynamic_lookup"
            ];
          });

      defaultPkg = pkgs.stdenv.mkDerivation {
        src = ./.;
        name = "install";
        installPhase = let
          extension =
            if system == "aarch64-darwin"
            then "dylib"
            else "so";
          # Note, "so" is always the desired output extension, even on MacOS
        in ''
          mkdir -p $out/lua
          ls ${ned}/lib
          cp -r ${ned}/lib/libned.${extension} $out/lua/ned.so

        '';
      };
    in {
      devShells.default = pkgs.mkShell {
        # buildInputs = with pkgs; lib.lists.optionals stdenv.isDarwin [pkgs.libiconv];
        nativeBuildInputs = with pkgs;
          [
            pkg-config
            luajit
            lua
            gnumake
            gcc
            clang

            libiconv
            # toolchain
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.buildPlatform.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.CoreFoundation
            pkgs.darwin.apple_sdk.frameworks.CoreServices
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            pkgs.libiconv
          ];
      };

      packages.default = defaultPkg;
      apps.default = {
        type = "app";
        program = "${(
          pkgs.writers.writeBashBin "b" ''
            rm -rf ./lua
            mkdir -p lua
            cp ${defaultPkg}/lua/ned.so ./lua/ned.so
          ''
        )}/bin/b";
      };
    });
}
