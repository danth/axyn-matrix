packages:
{ pkgs, config, lib, ... }:

with lib;

let
  cfg = config.services.axyn-matrix;

in {
  options.services.axyn-matrix = {
    enable = mkEnableOption "Axyn for Matrix";

    homeserver = mkOption {
      description = "URL of the Matrix homeserver.";
      type = types.str;
      example = "https://matrix.org";
    };

    username = mkOption {
      description = "Username to log in to the homeserver with.";
      type = types.str;
      example = "axyn";
    };

    password = mkOption {
      description = "Password to log in to the homeserver with.";
      type = types.str;
    };

    passwordScript = mkOption {
      description = ''
        Shell script which prints the password to stdout.

        This can be used to load the password from a file, or to register the
        account with the homeserver on first startup.
      '';
      type = types.lines;
      default = "echo ${escapeShellArg cfg.password}";
      example = "cat /run/secrets/axyn-password";
    };
  };

  config = {
    systemd.services.axyn-matrix = {
      description = "Axyn for Matrix";
      documentation = [ "https://github.com/danth/axyn-matrix#readme" ];

      wantedBy = [ "default.target" ];
      after =
        optional config.services.matrix-synapse.enable "matrix-synapse.service" ++
        optional config.services.matrix-conduit.enable "conduit.service";

      serviceConfig = {
        DynamicUser = true;
        User = "axyn-matrix";
        StateDirectory = "axyn-matrix";
        Restart = "on-failure";
      };

      path = with pkgs; [ coreutils packages.${system}.default ];
      script = ''
        password="$({
          ${cfg.passwordScript}
        })"

        if [ -f /var/lib/axyn-matrix/device-id ]; then
          deviceId="$(cat /var/lib/axyn-matrix/device-id)"
        else
          deviceId="$(tr -dc A-Z </dev/urandom | head -c 10)"
          echo -n "$deviceId" >/var/lib/axyn-matrix/device-id
        fi

        export HOME=/var/lib/axyn-matrix

        axyn ${cfg.homeserver} ${cfg.username} "$password" "$deviceId"
      '';
    };
  };
}
