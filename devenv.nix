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
  ++ lib.optionals (!config.containers.prod.isBuilding) [ pkgs.git pkgs.watchexec pkgs.earthly ];

  enterTest = ''
    cargo test
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
    # Watch the package.json file, if that changes, restart everything from installation.
    # Watch the source tree, if that changes, rebuild everything.
    backend.exec = "htmx-watch";
    build.exec = "cargo build --release";
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
    rust.channel = "stable";
    javascript.enable = true;
    javascript.pnpm.enable = true;
  };

  pre-commit.hooks = {
    clippy.enable = true;
    clippy.settings.allFeatures = true;
  };

  containers.prod = {
    copyToRoot = [
      ./frontend
      ./config
    ];
    startupCommand = "./bookclub";
  };
}
