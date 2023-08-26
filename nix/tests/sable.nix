{ pkgs, sableModule }:

let
  inherit (pkgs) lib system;

  deriveFingerprint = certFile: builtins.readFile (pkgs.runCommand "openssl-get-sha1-fp" ''
    ${lib.getExe pkgs.openssl} x509 -in ${certFile} -fingerprint -noout | sed 's/.*=//;s/://g;s/.*/Ł&/' > $out
  '');
  nodeConfig = { serverId, serverName, managementHostPort ? 8080, serverIP, caFile, managementClientCaFile, keyFile, certFile
    , bootstrapped ? false, peers ? null }: { ... }:
    let myFingerprint = ""; # deriveFingerprint certFile; # compute it
    in {
      imports = [ 
        sableModule
      ];

      networking.interfaces.eth1.ipv4.addresses = [
        { address = serverIP; prefixLength = 24; }
      ];

      networking.firewall.allowedTCPPorts = [ 6667 6668 6697 8888 ];

      virtualisation.forwardPorts = [{
        host.port = managementHostPort;
        guest.port = 8888;
      }];

      services.sable = {
        enable = true;
        package = pkgs.sable-dev;
        network.bootstrap = if bootstrapped then
          (lib.importJSON ../../configs/network_config.json)
        else
          null;
        network.settings = {
          fanout = 1;
          ca_file = caFile;
          peers = if (peers != null) then
            peers
          else ([{
            name = serverName;
            address = "${serverIP}:6668";
            fingerprint = myFingerprint;
          }]);
        };
        server.settings = let
          keys = {
            key_file = keyFile;
            cert_file = certFile;
          };
        in {
          server_id = serverId;
          server_name = serverName;

          management = {
            address = "127.0.0.1:8888";
            client_ca = managementClientCaFile;
            authorised_fingerprints = [{
              name = "user1";
              fingerprint = "435bc6db9f22e84ba5d9652432154617c9509370";
            }];
          };

          server.listeners = [
            { address = "${serverIP}:6667"; }
            {
              address = "${serverIP}:6697";
              tls = true;
            }
          ];

          tls_config = keys;
          node_config = keys // { listen_addr = "${serverIP}:6668"; };
        };
      };
    };

  mkMultiNodeTest = { name, servers ? { }, client ? { }, testScript }:
    pkgs.nixosTest {
      inherit name testScript;
      nodes = servers // {
        client = { lib, ... }: {
        imports = [ sableModule client ];

        systemd.services.weechat-headless = {
          serviceConfig.StateDirectory = "weechat";
          script = ''
            ${pkgs.weechat}/bin/weechat-headless --stdout -d /var/lib/weechat --run "/logger set 9; /set fifo.file.path /tmp/weechat_fifo; /plugin unload fifo; /plugin load fifo; /fifo enable; /logger set 9"'';
          wantedBy = [ "multi-user.target" ];
          wants = [ "sable-ircd.service" ];
          after = [ "sable-ircd.service" ];
        };
      };
    };
  };

  mkBasicTest = { name, machine ? { }, testScript }:
    pkgs.nixosTest {
      inherit name testScript;
      nodes.machine = { lib, ... }: {
        imports = [
          sableModule
          (nodeConfig {
            serverId = 1;
            serverIP = "127.0.1.2";
            serverName = "server1.test";
            bootstrapped = true;
            caFile = ../../configs/ca_cert.pem;
            managementClientCaFile = ../../configs/ca_cert.pem;
            keyFile = ../../configs/server1.key;
            certFile = ../../configs/server1.pem;
          })
          machine
        ];

        systemd.services.weechat-headless = {
          serviceConfig.StateDirectory = "weechat";
          script = ''
            ${pkgs.weechat}/bin/weechat-headless --stdout -d /var/lib/weechat --run "/logger set 9; /set fifo.file.path /tmp/weechat_fifo; /plugin unload fifo; /plugin load fifo; /fifo enable; /logger set 9"'';
          wantedBy = [ "multi-user.target" ];
          wants = [ "sable-ircd.service" ];
          after = [ "sable-ircd.service" ];
        };
      };
    };
in {
  monoNode = mkBasicTest {
    name = "mononode-sable";
    testScript = ''
      machine.start()
      machine.wait_for_unit("sable-ircd.service")
      machine.wait_for_unit("weechat-headless.service")

      def remote_weechat(command: str):
        return machine.succeed(f"echo \"{command}\" > /tmp/weechat_fifo")

      print(machine.succeed("sleep 1"))
      remote_weechat(" */server add test-ircd-nossl 127.0.1.2/6667")
      remote_weechat(" */connect test-ircd-nossl")
      remote_weechat("irc.server.test-ircd-nossl */nick test")
      remote_weechat("irc.server.test-ircd-nossl */join #hello")
      remote_weechat("irc.test-ircd-nossl.#hello *Hello world!")
    '';
  };

  basicMultiNodes = 
  let
    peers = [
      {
        name = "server1.test";
        address = "192.168.1.10:6668";
        fingerprint = "961090b178e037be12a77c0a83876740e3222abd";
      }
      {
        name = "server2.test";
        address = "192.168.1.11:6668";
        fingerprint = "000fd90df3da5619563eb49228229ac410acc09c";
      }
    ];
    servers = {
      server1 = nodeConfig {
        serverId = 1;
        serverName = "server1.test";
        bootstrapped = true;
        serverIP = "192.168.1.10";
        caFile = ../../configs/ca_cert.pem;
        managementClientCaFile = ../../configs/ca_cert.pem;
        keyFile = ../../configs/server1.key;
        certFile = ../../configs/server1.pem;
        managementHostPort = 8080;
        inherit peers;
      };
      server2 = nodeConfig {
        serverId = 2;
        serverName = "server2.test";
        bootstrapped = false;
        serverIP = "192.168.1.11";
        caFile = ../../configs/ca_cert.pem;
        managementClientCaFile = ../../configs/ca_cert.pem;
        keyFile = ../../configs/server2.key;
        certFile = ../../configs/server2.pem;
        managementHostPort = 8081;
        inherit peers;
      };
    };
    serverNames = [ "server1" "server2" ];
  in
  mkMultiNodeTest {
    name = "basic-multinodes-sable";
    inherit servers;
    testScript = ''
      import json
      start_all()

      servers = [globals()[server_name] for server_name in [${lib.concatMapStringsSep ", " (name: "\"${name}\"") serverNames}]]

      for server in servers:
        server.wait_for_unit("sable-ircd.service")

      def remote_weechat(command: str):
        return client.succeed(f"echo \"{command}\" > /tmp/weechat_fifo")

      def test_server(name: str, server: str, nick: str = "test", channel_to_join: str = "hello"):
        remote_weechat(f" */server add {name} {server}/6667")
        remote_weechat(f" */connect {name}")
        remote_weechat(f"irc.server.{name} */nick {nick}")
        remote_weechat(f"irc.server.{name} */join #{channel_to_join}")

      def get_network_state(server):
        # For now, certificate expired, so -k is necessary.
        intermediate = json.loads(server.succeed("curl -k --capath ${../../configs/ca_cert.pem} --cert ${../../configs/mgmt.pem} --key ${../../configs/mgmt.key} https://localhost:8888/dump-network"))
        # Fix up 'servers' which are HashMap<Id, Type>
        intermediate['servers'] = {k: v for (k, v) in intermediate['servers']}
        return intermediate

      # This server has a mgmt endpoint because it's bootstrapped
      # The other not yet until sync.
      test_server("server-a", "server1", "test-from-a")
      # Wait for synchronization.
      server2.wait_for_open_port(8888)
      test_server("server-b", "server1", "test-from-b")
      nstate_a = get_network_state(server1)
      nstate_b = get_network_state(server2)
      print(nstate_a)
      print(nstate_b)

      assert nstate_a == nstate_b, "Network state diverged"
    '';
  };
}
