project_root := justfile_directory()

show_flake_outputs:
  om show {{project_root}}

install:
  just install_cargo_tool wrpc
  # After it's released, will use flox to install
  just install_cargo_tool dioxus-cli@0.7.0-rc.1 --force

install_cargo_tool *ARGS:
  cargo install --root {{project_root}} {{ARGS}}

