{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/basics/
  env.GREET = "devenv";
  env.PODMAN_IGNORE_CGROUPSV1_WARNING = "1";

  # https://devenv.sh/packages/
  packages = with pkgs ; [
    git
    grafana
    podman
    duckdb
    ockam
    erg
    python3
  ];

  # https://devenv.sh/languages/
  languages.rust.enable = true;

  # https://devenv.sh/processes/
  process.managers.process-compose.tui.enable = true;

  #processes.grafana.exec = "";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/scripts/
  scripts.hello.exec = ''
    echo hello from $GREET
  '';

  enterShell = ''
    hello
    git --version
  '';

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep --color=auto "${pkgs.git.version}"
  '';

  # https://devenv.sh/pre-commit-hooks/
  # pre-commit.hooks.shellcheck.enable = true;

  # See full reference at https://devenv.sh/reference/options/
}
