# Sable, a chat server

Sable is an experimental chat server designed to address many of the fundamental limitations of legacy
IRC server software. In particular:

 * Server linking and network state tracking are completely separated from client protocol mechanics.
 * Server nodes communicate using a Gossip-like protocol, removing the need for a spanning-tree routing
   and making netsplits, as they occur in legacy servers, a thing of the past. While a server may still
   become isolated from the network due to networking faults, the loss of a single connection will not
   disrupt the network.
 * All network state events are uniquely identified, and dependencies between them are tracked. Despite
   the arbitrarily-connected Gossip network, duplicate or out-of-order processing of events will not
   occur.
 * Complete network state history allows persistent user presence. Users can remain online when their
   clients disconnect, and can have multiple client connections, potentially on different servers, to
   a single user session.
 * Instead of dynamic module loading and all the complications it can cause, server code upgrades are
   handled by re-executing the server in-place and resuming from saved state information. A further
   advantage is that there is no 'core' that cannot be upgraded without visible disruption.

# Architecture

Things are still changing too much to draw a diagram here. The foundational principles are:

 * Every network object (user, channel, channel topic, channel ban, etc.) has a unique identifier.
 * Every change to network state is represented by an `Event`. All events have a target object which
   they modify, a globally-unique event ID, an event clock containing its dependency information, other
   metadata, and a typed details structure describing the change being made.
 * Events are propagated between servers via a gossip protocol.
 * Each server maintains an event log. An incoming event is added to the log only when all its
   dependencies have been added and processed. If an incoming event has missing dependencies, they are
   requested from the network.
 * A server's view of network state is only ever updated by processing `Event`s emitted by the event log.
   Event application is careful to ensure that any valid order of application for a set of events will
   eventually produce the same state.
 * All other code runs with a read-only view of network state. If updates need to be made, this is done
   by emitting an event for propagation and processing by the event log.
 * There is no such thing as a netjoin. When a server starts up, it will not process any client
   connections until it has synced to a network, unless it has been specifically told to bootstrap a new
   network. If a server becomes completely detached from the network and cannot exchange events, it will
   be disconnected from the network permanently until it restarts.

Some other design decisions which are not as fundamental but are worth mentioning:

 * Client listeners and DNS/identd connections are farmed out into a separate process to make seamless
   code upgrades easier.
 * Server and network maintenance operations are performed via an out-of-band HTTP management service,
   not via IRC client connections.

## Navigating the code

Currently, Sable is split into several crates:

  * `sable_network` contains all of the network data model and logic for running a network server. This
    includes state tracking but no client protocol logic.
    * `sable_network::network` contains the `Network` type which represents network state and handles
      event application and conflict resolution, as well as the various state objects and convenience wrappers
      that make up a network. It also contains definitions of the event types.
    * `sable_network::history` contains the `NetworkHistoryLog` type which represents the network history
      as it is visible to each user of the network
    * `sable_network::sync` contains the network synchronisation code
    * `sable_network::server` contains the basic network server and state management
  * `sable_ircd` contains the IRC client server
  * `client_listener` and `auth_client` contain split-out parts of the client server which run in their
    own processes.
  * `sable_ipc` contains IPC channel types used by the split-out processes to communicate with the main
    client server.
  * `sable_macros` contains procedural macros used by the other crates.

# Building Sable

1. Install rust. [The rust people say you should use rustup.](https://www.rust-lang.org/tools/install)
2. Clone this repo.
3. `cargo build`

# Running Sable

There's a sample set of configs and certificates in the `configs` directory, which will run a network of
two servers on 127.0.1.2 and 127.0.1.3, both using 6667 and 6697 for client connections, and 6668 for server
linking. To run them:

```
$ ./target/debug/ircd -n configs/network.conf -s configs/server1.conf --bootstrap-network configs/network_config.json
$ ./target/debug/ircd -n configs/network.conf -s configs/server2.conf
```

There are two types of network configuration. At present, the list of nodes and their network addresses
is static, and read only at startup. This is what's in the `network.conf` file, which should be shared
between all server nodes. Runtime configuration currently consists of operator credentials, and is
contained in a separate file (`network_config.json` in the examples). This can be loaded via the command
line when bootstrapping a new network, or updated at runtime via the `config_loader` utility.