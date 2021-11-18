{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/7e567a3d092b7de69cdf5deaeb8d9526de230916";
    utils.url = "github:numtide/flake-utils";
    flipgenic = {
      url = "github:danth/flipgenic";
      flake = false;
    };
  };
  outputs = inputs@{ nixpkgs, utils, ... }:
    utils.lib.eachSystem ["x86_64-linux"]
    (system:
      with nixpkgs.legacyPackages.${system}.python3Packages;
      let
        pkgs = nixpkgs.legacyPackages.${system};

        ngt = buildPythonPackage rec {
          pname = "ngt";
          version = "v1.12.3";
          src = pkgs.fetchFromGitHub {
            owner = "yahoojapan";
            repo = "NGT";
            rev = version;
            sha256 = "d2DUnuSlnMhd/QDJZRJLXQvcat37dEM+s9Ci4KXxxvQ=";
          };
          postPatch = ''
            substituteInPlace python/src/ngtpy.cpp \
              --replace "NGT_VERSION" '"${version}"'
          '';
          preConfigure = ''
            export HOME=$PWD
            export LD_LIBRARY_PATH=${pkgs.ngt}/lib
            cd python
          '';
          buildInputs = [ pkgs.ngt ];
          propagatedBuildInputs = [ numpy pybind11 ];
        };

        en-core-web-md = buildPythonPackage rec {
          pname = "en-core-web-md";
          version = "3.0.0";
          src = pkgs.fetchzip {
            url =
              "https://github.com/explosion/spacy-models/releases/download/en_core_web_md-3.0.0/en_core_web_md-3.0.0.tar.gz";
            sha256 = "4UrUhHNVLHxbOdm3BIIetv4Pk86GzFoKoSnlvLFqesI=";
          };
          propagatedBuildInputs = [ spacy ];
        };

        flipgenic = buildPythonPackage rec {
          name = "flipgenic";
          src = inputs.flipgenic;
          propagatedBuildInputs = [ ngt spacy sqlalchemy ];
        };

        initial-responder = pkgs.callPackage ./initial_responder {
          inherit en-core-web-md flipgenic;
        };

        axyn-matrix = buildPythonApplication rec {
          name = "axyn-matrix";
          src = ./.;
          propagatedBuildInputs = [
            en-core-web-md
            flipgenic
            matrix-nio
          ];
          postPatch = ''
            substituteInPlace axyn_matrix/__main__.py \
              --replace '@INITIAL_RESPONDER@' '${initial-responder}'
          '';
        };

        axyn-matrix-app = utils.lib.mkApp {
          drv = axyn-matrix;
          exePath = "/bin/axyn_matrix";
        };

      in {
        packages = { inherit axyn-matrix; };
        defaultPackage = axyn-matrix;

        apps.axyn-matrix = axyn-matrix-app;
        defaultApp = axyn-matrix-app;
      });
}
