{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
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

        ngtVersion = "1.14.4";
        ngtSrc = pkgs.fetchFromGitHub {
          owner = "yahoojapan";
          repo = "NGT";
          rev = "v${ngtVersion}";
          sha256 = "Wa15AdLLzOL7qqCR2Bt+0i0YIgd6qKBmtEU9Q9/uiQI=";
        };

        ngt = pkgs.stdenv.mkDerivation rec {
          pname = "NGT";
          version = ngtVersion;
          src = ngtSrc;

          nativeBuildInputs = with pkgs; [ cmake ];
          buildInputs = with pkgs; [ llvmPackages.openmp ];

          __AVX2__ = 0;
        };

        ngtpy = buildPythonPackage rec {
          pname = "NGTpy";
          version = ngtVersion;
          src = ngtSrc;

          postPatch = ''
            substituteInPlace python/src/ngtpy.cpp \
              --replace "NGT_VERSION" '"${version}"'
          '';

          preConfigure = ''
            export HOME=$PWD
            export LD_LIBRARY_PATH=${ngt}/lib
            cd python
          '';

          buildInputs = [ ngt ];
          propagatedBuildInputs = [ numpy pybind11 ];
        };

        en-core-web-md = buildPythonPackage rec {
          pname = "en-core-web-md";
          version = "3.3.0";
          src = pkgs.fetchzip {
            url =
              "https://github.com/explosion/spacy-models/releases/download/en_core_web_md-3.3.0/en_core_web_md-3.3.0.tar.gz";
            sha256 = "sy+vff9STk6vBlWybdbsVL9+orANuUNJjma1ar+u30s=";
          };
          propagatedBuildInputs = [ spacy ];
        };

        flipgenic = buildPythonPackage rec {
          name = "flipgenic";
          src = inputs.flipgenic;
          propagatedBuildInputs = [ ngtpy spacy sqlalchemy ];
        };

        initial-responder = pkgs.callPackage ./initial_responder {
          inherit en-core-web-md flipgenic;
        };

        axyn-matrix = buildPythonApplication rec {
          name = "axyn-matrix";
          src = ./.;
          propagatedBuildInputs = [
            aiofiles
            beautifulsoup4
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
