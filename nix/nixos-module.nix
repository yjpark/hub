# The hub daemon as a NixOS system service.
#
# The hub routes and never owns: it keeps no state beyond liveness, so the unit
# needs no state directory and runs as a dynamic user.
{ config, lib, ... }:
let
  cfg = config.services.hub;
in
{
  options.services.hub = {
    enable = lib.mkEnableOption "the hub daemon";

    package = lib.mkOption {
      type = lib.types.package;
      description = "Package providing the `hub` binary.";
    };

    address = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1:15400";
      example = "10.100.0.1:15400";
      description = "Address to bind; nodes reverse-dial it.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.hub = {
      description = "hub — routes calls between nodes and consumers";
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      environment.HUB_ADDR = cfg.address;
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/hub";
        Restart = "always";
        RestartSec = 2;
        DynamicUser = true;
      };
      # Binding a bridge address can race the bridge coming up; keep retrying.
      startLimitIntervalSec = 0;
    };
  };
}
