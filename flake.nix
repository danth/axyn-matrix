{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    utils.url = "github:numtide/flake-utils";
  };
  outputs = inputs@{ nixpkgs, utils, ... }:
    utils.lib.eachSystem ["x86_64-linux"]
    (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in { });
}
