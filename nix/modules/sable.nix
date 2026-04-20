{ lib, config, pkgs, ... }:
with lib;
let
  cfg = config.services.sable;

  settingsFormat = pkgs.formats.json { };
  serverConfigFile = settingsFormat.generate "server.json" cfg.server.settings;
  networkConfigFile =
    settingsFormat.generate "network.json" cfg.network.settings;
  bootstrapFile =
    settingsFormat.generate "bootstrap.json" cfg.network.bootstrap;
in {
  options.services.sable = {
    enable = mkEnableOption "the Libera Chat's Sable IRCd";

    package = mkPackageOptionMD pkgs "sable" {};

    network = {
      configFile = mkOption {
        type = types.path;
        default = networkConfigFile;
      };

      settings = mkOption rec {
        type = types.submodule { freeformType = settingsFormat.type; };

        default = { };
      };

      bootstrap = mkOption {
        type = types.nullOr (types.submodule { freeformType = settingsFormat.type; });
        default = null;
      };
    };

    server.configFile = mkOption {
      type = types.path;
      default = serverConfigFile;
    };

    server.settings = mkOption {
      type = types.submodule { freeformType = settingsFormat.type; };

      default = { };

      description = ''
        Server configuration of the IRCd.
        '';
    };
  };

  config = mkIf cfg.enable {
    services.sable.server.settings.log = lib.mapAttrs (n: lib.mkDefault) {
      dir = "/var/log/sable/";
      # stdout = "stdout.log";
      # stderr = "stderr.log";
      pidfile = "/run/sable/sable.pid";
      module-levels = {
        tokio = "trace";
        runtime = "trace";
        rustls = "error";
        tracing = "warn";
        sable = "trace";
        "" = "debug";
      };

      targets = [
        {
          target = "stdout";
          level = "trace";
          modules = [ "sable"];
        }
        {
          target = { filename = "sable.log"; level = "info"; };
          level = "info";
        }
        {
          target = { filename = "trace.log"; level = "trace"; };
          level = "trace";
        }
      ];

      console-address = "127.0.0.1:9999";
    };
    systemd.services.sable-ircd = {
      description = "Sable IRC daemon";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      restartTriggers = [ cfg.network.configFile cfg.server.configFile ];
      serviceConfig = {
        LogsDirectory = "sable";
        PIDFile = "sable.pid";
        ExecStart = "${cfg.package}/bin/sable_ircd --foreground ${optionalString (cfg.network.bootstrap != null) "--bootstrap-network ${bootstrapFile}"} --network-conf ${cfg.network.configFile} --server-conf ${cfg.server.configFile}";
      };
    };
  };
}
