{ pkgs, lib, config, inputs, ... }:
let
  frontendDir = builtins.toString ./frontend;
in
{
  # https://devenv.sh/packages/
  packages = [
    pkgs.git
    pkgs.libiconv
    pkgs.watchexec
  ] ++ lib.optionals (pkgs.stdenv.isDarwin) [ pkgs.darwin.apple_sdk.frameworks.Foundation ];

  scripts.backend-watch.exec = ''
    watchexec \
      --ignore-file frontend/.gitignore \
      --ignore '*.nix' \
      --restart \
      'cd frontend && pnpm run build && cd - && cargo loco start --binding 127.0.0.1' 
  '';
  # https://devenv.sh/scripts/
  scripts.backend.exec = ''
    watchexec \
      --watch frontend/package.json \
      --restart \
      'cd frontend && pnpm install && cd - && backend-watch'
  '';

  enterShell = ''
  '';

  # https://devenv.sh/tests/
  enterTest = ''
    cargo test
  '';

  processes.backend.exec = "backend";
  processes.frontend.exec = "cd frontend && pnpm run dev";

  # https://devenv.sh/services/
  services.postgres = {
    enable = true;
    package = pkgs.postgresql_16;
    listen_addresses = "127.0.0.1";
    initialDatabases = [{ name = "bookclub_development"; }];
    initialScript = ''
      CREATE ROLE loco SUPERUSER LOGIN PASSWORD 'loco';
    '';
  };
  env.DATABASE_URL = "postgres://loco:loco@127.0.0.1/bookclub_development";
  services.redis = {
    enable = true;
  };

  # https://devenv.sh/languages/
  languages.nix.enable = true;
  languages.rust = {
    enable = true;
    channel = "stable";
  };
  languages.javascript = {
    enable = true;
    pnpm.enable = true;
  };

  # https://devenv.sh/pre-commit-hooks/
  # pre-commit.hooks.shellcheck.enable = true;

  # https://devenv.sh/processes/
  # processes.ping.exec = "ping example.com";

  # See full reference at https://devenv.sh/reference/options/
}
