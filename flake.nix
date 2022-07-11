{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nix-filter.url = "github:numtide/nix-filter";

    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self, nixpkgs, crane, nix-filter, utils, ... }:
    {
      nixosModules.axyn = import ./nixos.nix self.packages;
    } //
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        craneLib = crane.lib.${system};

        fasttext-wiki-news-subword = pkgs.fetchzip {
          name = "wiki-news-300d-1M-subword.vec";
          url = "https://dl.fbaipublicfiles.com/fasttext/vectors-english/wiki-news-300d-1M-subword.vec.zip";
          sha256 = "e5WZ7gMZP3PvJOXEbP4bOx36oUqaTBt+7PrfkVso6lU=";
        };

        commonArguments = rec {
          src = nix-filter.lib {
            root = ./.;
            include = with nix-filter.lib; [
              "Cargo.toml"
              "Cargo.lock"
              (inDirectory "src")
            ];
          };

          cargoArtifacts = craneLib.buildDepsOnly {
            inherit src;
            buildInputs = with pkgs; [ pkg-config openssl.dev ];
          };

          WORD2VEC_DATA = "${fasttext-wiki-news-subword}/wiki-news-300d-1M-subword.vec";
        };

      in {
        packages.default = craneLib.buildPackage commonArguments;

        checks.clippy = craneLib.cargoClippy (commonArguments // {
          cargoClippyExtraArgs = "-- --deny warnings";
        });

        devShells.default = with pkgs; mkShell {
          nativeBuildInputs = [
            cargo
            (writeShellScriptBin "rustfmt" ''
              PATH=${rustfmt.override { asNightly = true; }}/bin
              rustfmt src/*.rs
            '')
          ];
        };
      }
    );
}
