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

    extraAddresses = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      example = [ "172.22.1.9:15400" ];
      description = "More addresses serving the same hub — e.g. a private network peer hubs dial.";
    };

    name = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "pc";
      description = "This hub's name, as its peers qualify its nodes (`pc::yolo`); null means the hostname.";
    };

    peers = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      example = { edger = "172.22.1.2:15400"; };
      description = "Peer hubs by name and address; their nodes are reached one hop away as `<name>::<node>`.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.hub = {
      description = "hub — routes calls between nodes and consumers";
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      environment = {
        HUB_ADDR = lib.concatStringsSep "," ([ cfg.address ] ++ cfg.extraAddresses);
      } // lib.optionalAttrs (cfg.name != null) { HUB_NAME = cfg.name; }
      // lib.optionalAttrs (cfg.peers != { }) {
        HUB_PEERS = lib.concatStringsSep "," (lib.mapAttrsToList (n: a: "${n}=${a}") cfg.peers);
      };
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
