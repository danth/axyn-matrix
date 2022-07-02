{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self, nixpkgs, crane, utils, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        craneLib = crane.lib.${system};
        
        glove-twitter = pkgs.fetchzip {
          name = "glove-twitter-27B";
          url = "https://nlp.stanford.edu/data/wordvecs/glove.twitter.27B.zip";
          stripRoot = false;
          sha256 = "5hoiV2Aiould/WZctpkLzQ99PUzp+pkFQCqOiZcrT4g=";
        };

        cargoArtifacts = craneLib.buildDepsOnly {
          src = ./.;
        };

        package = craneLib.buildPackage {
          src = ./.;
          inherit cargoArtifacts;
          WORD2VEC_DATA = "${glove-twitter}/glove.twitter.27B.200d.txt";
        };

      in {
        packages.default = package;
        apps.default = utils.lib.mkApp { drv = package; };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ cargo rustc ];
        };
      }
    );
}
