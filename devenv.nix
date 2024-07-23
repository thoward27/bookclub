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
    cargo loco task seed_data
  '';

  scripts = {
    # This script was extracted to reduce the double '' needed to nest the watchexec commands.
    backend-watch.exec = ''
      watchexec \
        --ignore-file frontend/.gitignore \
        --ignore '*.nix' \
        --restart \
        'cd frontend && pnpm run build && cd - && cargo loco start --binding 127.0.0.1' 
    '';
    # Install is broken out because when it is run in the backend-watch it doom-cycles.
    pnpm-install-watch.exec = ''
    watchexec \
      --watch frontend/package.json \
      --restart \
      'cd frontend && pnpm install && cd - && backend-watch'
    '';
    htmx-watch.exec = ''
      watchexec \
        --watch templates \
        --watch src \
        --restart \
        cargo loco start --binding 127.0.0.1
    '';
  };

  processes = {
    backend.exec = "htmx-watch";
  };

  services = {
    postgres = {
      enable = true;
      package = pkgs.postgresql_16;
      listen_addresses = "127.0.0.1";
      initialDatabases = [{ name = "bookclub_development"; }];
      initialScript = ''
        CREATE ROLE loco SUPERUSER LOGIN PASSWORD 'loco';
      '';
    };
    redis = {
      enable = true;
    };
  };

  env.DATABASE_URL = "postgres://loco:loco@127.0.0.1/bookclub_development";

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
