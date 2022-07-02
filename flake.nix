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

        nonRustDependencies = {
          buildInputs = with pkgs; [ ];
          nativeBuildInputs = with pkgs; [ ];
        };

        cargoArtifacts = craneLib.buildDepsOnly ({
          src = ./.;
        } // nonRustDependencies);

        package = craneLib.buildPackage ({
          src = ./.;
          inherit cargoArtifacts;
        } // nonRustDependencies);

      in {
        packages.default = package;
        apps.default = utils.lib.mkApp { drv = package; };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ cargo rustc ];
        };
      }
    );
}
