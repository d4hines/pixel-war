{
  packages,
  tezos,
}: {
  config,
  pkgs,
  lib,
  ...
}:
with lib; let
  cfg = config.services.tezos-place;
  listToString = lib.strings.concatStringsSep ",";
  myPkgs = packages.${config.nixpkgs.system};
in {
  options.services.tezos-place = {
    enable = mkEnableOption "tezos-place system";
  };

  config = mkIf cfg.enable {
    networking.firewall = {
      allowedTCPPorts = [
        8080
      ];
    };
    systemd = {
      timers = {
        tezos-place-process-messages = {
          description = "Tezos Place - process messages timer";
          wantedBy = [ "timers.target" ];
          timerConfig.OnCalendar = "*-*-* *:*:00";
        };
      };
      services = {
        tezos-place-process-messages = {
          description = "Tezos Place - process messages";
          serviceConfig.Type = "simple";
          serviceConfig.ExecStart = "${myPkgs.process-message}/bin/process-message";
          serviceConfig.User = "d4hines";
          wantedBy = ["default.target"];
        };
        tezos-place-sequencer = {
          description = "Tezos Place Sequencer";
          after = ["network.target"];
          wantedBy = ["multi-user.target"];
          path = [];
          environment = {
            SEQUENCER_SECRET_KEY = "${builtins.readFile ../secret/sequencer_key}";
            ROLLUP_ADDRESS = "http://localhost:8932";
            ROLLUP_PREIMAGES_DIR = "/var/lib/rollup/.tezos-smart-rollup-node/wasm_2_0_0";
            ROLLUP_EXTERNAL_MESSAGE_LOG = "/var/lib/tezos-place/external_message_log";
            ROLLUP_TX_LOG = "/var/lib/tezos-place/tx_log";
            ROLLUP_IMAGE = "/var/lib/tezos-place/image.png";
            ROLLUP_MESSAGE_INDEX = "/var/lib/tezos-place/next_index";
            TZPLACE_FRONTEND = "${myPkgs.frontend}/lib/node_modules/frontend/dist";
          };
          serviceConfig = {
            Type = "simple";
            ExecStart = "${myPkgs.sequencer}/bin/sequencer";
            Restart = "on-failure";
            StateDirectory = "tezos-place";
            RuntimeDirectory = "tezos-place";
            RuntimeDirectoryPreserve = "yes";
            MemoryMax = "155G";
            # Basic Hardening
            # NoNewPrivileges = "yes";
            # PrivateTmp = "yes";
            # PrivateDevices = "yes";
            # DevicePolicy = "closed";
            # DynamicUser = "true";
            # ProtectSystem = "strict";
            # ProtectHome = "read-only";
            # ProtectControlGroups = "yes";
            # ProtectKernelModules = "yes";
            # ProtectKernelTunables = "yes";
            # RestrictAddressFamilies = "AF_UNIX AF_INET AF_INET6 AF_NETLINK";
            # RestrictNamespaces = "yes";
            # RestrictRealtime = "yes";
            # RestrictSUIDSGID = "yes";
            # MemoryDenyWriteExecute = "no";
            # LockPersonality = "yes";
          };
        };
        tezos-place-rollup-node = {
          description = "Tezos Place Rollup Node";
          after = ["network.target"];
          wantedBy = ["multi-user.target" "tezos-place-sequencer.service"];
          requires = ["tezos-place-sequencer.service"];
          path = [];
          environment = {
            TEZOS_LOG = "* -> info";
            HOME = "/var/lib/rollup";
          };
          serviceConfig = {
            Type = "simple";
            ExecStart = let
              start-script = pkgs.writeShellApplication {
                name = "rollup-start";
                text = ''
                  exec ${tezos.packages.${config.nixpkgs.system}.trunk-octez-smart-rollup-node-PtMumbai}/bin/octez-smart-rollup-node-PtMumbai \
                    -E https://mainnet.api.tez.ie \
                    run \
                    --rollup sr1VHLzFsBdyL8jiEGHRdkBj3o9k7NujQhsx \
                    --mode observer \
                    --log-kernel-debug
                '';
              };
            in "${start-script}/bin/rollup-start";
            Restart = "on-failure";
            StateDirectory = "rollup";
            RuntimeDirectory = "rollup";
            RuntimeDirectoryPreserve = "yes";
          };
        };
      };
    };
  };
}
