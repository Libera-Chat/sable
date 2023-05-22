# Server Configuration

A sable network is made up of a number of nodes of different specialised types,
with common code and configuration for the shared aspects. The overall
configuration structure is identical for all node types.

## Top-Level Fields

* `server_id`: the numeric identifier for this node. This must be unique across
  the network at any given time, and should if possible be unique across the
  lifetime of the network.
* `server_name`: the textual name of the server; in most applications this would
  correspond to its DNS name, but this is not a requirement.

## `management`

The `management` block contains configuration for the HTTPS management
interface:

* `address`: The socket address (`IP:port`) for the management server to listen
  on.
* `client_ca`: The location of a (PEM-encoded) CA certificate file which will be
  used to validate client certificates.
* `authorised_fingerprints`: A list of authorised management users. Each item
  contains:
  * `name`: The username to be recorded for audit purposes
  * `fingerprint`: The fingerprint of the user's client certificate. Note that
    client certificates must be signed by the provided `client_ca` as well as
    matching the defined `fingerprint`.

## `tls_config`

This section defines the settings used for publicly-facing TLS connections, as
distinct from the server linking TLS settings which are defined in
`node_config`.

* `key_file`: The location of the (PEM-encoded) key file
* `cert_file`: The location of the (PEM-encoded) certificate file

## `node_config`

This section defines the settings used for server synchronisation.

* `listen_addr`: The socket address on which to listen for synchronisation
  messages. This must match the `address` defined for this server in the network
  configuration.
* `cert_file`: The location of a (PEM-encoded) certificate used to identify this
  node for synchronisation connections. This must be signed by the `ca_cert`
  defined in the network configuration, and its common name (CN) field must
  match the `server_name` field above.
* `key_file`: The location of the PEM-encoded private key for the server
  certificate.

## `log`

Logging configuration.

* `dir`: The parent directory in which to put log files. All other file names in
  the `log` section are relative to this directory.
* `stdout`: The file to which standard output is redirected, when running in
  background mode.
* `stderr`: The file to which standard error is redirected, when running in
  background mode.
* `pidfile`: The file in which to store the current process ID, when running in
  background mode.

* `module-levels`: A mapping of module names (as defined by the `tracing` crate)
  to the maximum level which should be logged for that module. An empty string
  as a module name defines the log level for all modules not otherwise defined.
* `targets`: An array of log target definitions, each of which contains:
  * `target`: the log target. This can be the special strings `stdout` or
    `stderr`, or a map containing the `filename` key.
  * `level`: The maximum level to include in this log target.
  * `modules`: An optional array of module names to include in this log target.
    If this is missing, then all modules will be included.

## `server`

The `server` block contains configuration items specific to each type of node.

### Client server configuration

The `server` block for a client server contains one item:

* `listeners`: an array of listener definitions, containing the following keys:
  * `address`: the listen address (`IP:port`)
  * `tls`: Option, if present and set to true then this is a TLS listener using
    the certificate and key defined in the global `tls_settings`

### Services configuration

The `server` block for a services node contains:

* `database`: the location of the account data store
* `default_roles`: a mapping of role names to arrays of permission items. When
  a new channel is registered, all of these default roles will be created for
  the new registration, and can be modified by the channel owner(s) later.