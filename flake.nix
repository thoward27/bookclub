{
  description = "Bookclub flake. A service to manage bookclubs.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        lib = nixpkgs.lib;
      in
      {
        packages = { };
        devShells.default = pkgs.mkShell {
          packages = [ pkgs.zsh pkgs.cargo pkgs.rustc pkgs.clippy pkgs.libiconv ];
          buildInputs = lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Foundation ];
          shellHook = ''
            exec ${pkgs.zsh}/bin/zsh -f
          '';
        };
      }
    );
}
