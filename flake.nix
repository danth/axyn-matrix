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

        package = craneLib.buildPackage commonArguments;

        clippy = craneLib.cargoClippy (commonArguments // {
          cargoClippyExtraArgs = "-- --deny warnings";
        });

      in {
        packages.default = package;
        apps.default = utils.lib.mkApp { drv = package; };

        checks = {
          inherit clippy;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ cargo rustc ];
        };
      }
    );
}
