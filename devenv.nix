# Docs: https://devenv.sh/basics/
{ pkgs, lib, config, inputs, ... }:
let
  frontendDir = builtins.toString ./frontend;
in
{
  name = "bookclub";

  packages = [
    pkgs.libiconv
  ]
  ++ lib.optionals (pkgs.stdenv.isDarwin) [ pkgs.darwin.apple_sdk.frameworks.Foundation ]
  ++ lib.optionals (!config.containers.prod.isBuilding) [ pkgs.git pkgs.watchexec pkgs.earthly pkgs.helix ];

  enterTest = ''
    cargo fmt --check
    cargo clippy
    cargo test
  '';

  scripts = {
    backend-watch.exec = ''
      watchexec \
        --ignore '*.nix' \
        --watch templates \
        --watch src \
        --restart \
        'cargo loco start --binding 127.0.0.1' 
    '';
  };

  processes.server.exec = "CARGO_TARGET_DIR=target/server backend-watch";

  services = {
    postgres = {
      enable = true;
      package = pkgs.postgresql_16;
      listen_addresses = "127.0.0.1";
      initialDatabases = [{ name = "bookclub_development"; }];
      initialScript = ''
        CREATE ROLE bookclub_prod SUPERUSER LOGIN PASSWORD 'loco';
      '';
    };
    redis = {
      enable = true;
    };
  };

  env.DATABASE_URL = "postgres://bookclub_prod:loco@127.0.0.1/bookclub_development";
  # This helps me in VSCode avoid having DevEnv Pre-Commit smash the cash from rust-analyzer.
  env.CARGO_TARGET_DIR = "target/devenv";

  languages = {
    nix.enable = true;
    rust.enable = true;
    rust.channel = "nightly";
    javascript.enable = true;
    javascript.pnpm.enable = true;
  };

  pre-commit.hooks = {
    clippy.enable = true;
    clippy.settings.allFeatures = true;
    clippy.settings.denyWarnings = true;
    cargo-check.enable = true;
    rustfmt.enable = true;
  };

  containers.prod = {
    copyToRoot = [
      ./frontend
      ./config
    ];
    startupCommand = "./bookclub";
  };
}
