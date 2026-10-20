{
  description = "hub — coordination layer for multi-agent development environments";

  inputs.jig.url = "github:edger-dev/jig";

  outputs = { self, jig }:
    jig.lib.mkWorkspace
      {
        pname = "hub";
        src = ./.;
        # extraDevPackages = pkgs: [ ];
      }
      {
        rust = {
          # buildPackages = [ "hub-cli" ];  # omit to build the whole workspace
          # wasm = true;                     # include the wasm32 target
        };
        kinora = {};   # puts the `kinora` CLI on the devShell PATH
      }
    // {
      # The hub daemon as a NixOS system service; see the module for options.
      nixosModules.hub = { lib, pkgs, ... }: {
        imports = [ ./nix/nixos-module.nix ];
        services.hub.package =
          lib.mkDefault self.packages.${pkgs.stdenv.hostPlatform.system}.hub;
      };
    };
}
