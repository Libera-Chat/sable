Building the thing
------------------

1. Install rust. [The rust people say you should use rustup.](https://www.rust-lang.org/tools/install)
2. Clone this repo.
3. `cargo build`

Running the thing
-----------------

There's a sample set of configs and certificates in the `configs` directory, which will run a network of
two servers on 127.0.1.2 and 127.0.1.3, both using 6667 and 6697 for client connections, and 6668 for server
linking. To run them:

```
$ ./target/debug/ircd -n configs/network.conf -s configs/server1.conf
$ ./target/debug/ircd -n configs/network.conf -s configs/server2.conf
```

There are two types of network configuration. At present, the list of nodes and their network addresses
is static, and read only at startup. This is what's in the `network.conf` file, which should be shared
between all server nodes. Runtime configuration currently consists of operator credentials, and needs to be
dynamically loaded once the network is running:

```
./target/debug/config_loader -n configs/network.conf -s configs/config.conf configs/network_config.json
```

