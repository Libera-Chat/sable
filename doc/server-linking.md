# Server Linking

Sable uses a Gossip-based protocol for server linking; every node in the network
communicates with every other node. If any particular pair of nodes are unable
to communicate directly, then messages will be routed via one or more other
nodes based upon what is reachable at the time.

## Network Configuration

The network configuration file must be replicated to all nodes on the network,
and must be the same for all nodes. A temporary mismatch between nodes while
updating configuration across the network can be recoverable, but message
delivery may be less reliable during this time.

The network configuration file contains the following settings:

 * `fanout`: the number of nodes to which each node will propagate each event.
   This setting should be tuned based on the total number of nodes and the
   desired trade-off of rapid delivery against bandwidth usage.
 * `ca_file`: the location of a (PEM-encoded) CA certificate which will be used
   to validate the TLS certificates of nodes participating in the sync network.
 * `peers`: an array of peer configurations (see below).

A peer configuration requires the following fields:

 * `name`: the server name. This must match the `server_name` field of that
   node's local configuration file.
 * `address`: the IP address and port of the node's synchronisation listener.
   This must match the `node_config.listen_addr` field of that server's
   configuration.
 * `fingerprint`: the fingerprint of the server's TLS certificate. Note that
   this is for the certificate used by the network sync listener, not the one
   used by any client listeners, which may be different.

## Peer Authentication

When an incoming connection is received on the network synchronisation listener,
the following must all be true in order for it to be accepted:

 * The client certificate must be signed by the CA provided in `ca_file`
 * The certificate common name must match the name of a server defined in the
   `peers` array
 * The certificate's fingerprint must match the `fingerprint` defined for that
   server
 * The source IP address must match the IP address portion of the `address`
   defined for that server

