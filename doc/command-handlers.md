# Writing Command Handlers

Command handlers are free functions, and take the various components they
require as function arguments, in a limited form of dependency injection. They
can be sync or async, and follow largely the same rules in either case, with a
small number of extra concerns for async handlers.

Command handlers are by convention located in submodules of
`sable_ircd/src/command/handlers/`, though they could in principle be located
anywhere in that crate. Files in that directory can provide illustrative
examples if required.

Handlers are identified by the `command_handler` attribute macro.

## The command_handler macro

The `command_handler` attribute macro can take several arguments:

```rs
#[command_handler("PRIMARY", "ALIAS", "ALIAS2", in("PARENT"), restricted)]
```

The first argument defines the primary name of the command. Any further strings
given will be used as aliases for the command.

If an argument is given of the form `in("PARENT")`, the command handler will be
put into a named secondary dispatcher, in this case `"PARENT"`. This form is
used to define handlers for services commands, and may have other uses in the
future. If this argument is not given, the handler is added to the 'default'
global dispatcher, which is used to look up client protocol commands.

If the `restricted` keyword is added, the command will be marked as for operators
and will not be shown in `HELP` output to users.

## Async handlers

Synchronous command handlers will finish execution before any other
command or network event is processed, and so do not need to worry about
concurrent modifications. Async handlers may be suspended at any await point,
at which time other commands or network events may be processed before the
handler is resumed.

The current approach to handling this is that the network state is copy-on-write
and the handler will by default operate on a view of the network state as it
was when the handler was first invoked. Any references to either the network
state or any object within it that are provided via dependency injection will
remain valid for the duration of the handler, but will always refer to the state
as it was at the beginning of the handler execution.

If a handler needs to access or respond to updated state at some point after it
was invoked, then it should:

 * Take object names, not parsed objects, as positional argument types
 * Not request a `Network` reference as an argument type, but instead take a
   `ClientServer` reference and access the network state via that type as
   required.
 * Release that returned network state before any await point, and request a new
   reference to network state each time it is required and might have changed.
 * Store object IDs, not object references, between accesses.

In most cases this should not be required; all relevant server and network state
objects are reference-counted and will remain valid for as long as a handler has
a reference to them.

The primary use case for asynchronous handlers is commands which may require a
remote request to a services or other special-purpose node. The request can be
sent, and the response awaited and acted upon, in a single function without
requiring a separate handler for the remote response as was required in legacy
platforms.

## Handler arguments

A handler function can take a number of 'ambient' parameters, followed by a
number of 'positional' parameters. Currently these are capped at six each, but
this might increase in future.

Ambient parameters are a form of limited dependency injection, and provide the
handler with access to the pieces of server and/or network state that it
requires in order to do its job.

Positional parameters are parsed directly from the arguments provided to the
command being processed.

There also exist conditional wrapper types for both positional and ambient
argument types which modify their behaviour in various ways.

### Ambient argument types

The following types can be used for ambient arguments:

 * `&dyn Command` - will provide a reference to an implementation of
   `command::Command` which describes the command being executed. This is most
   often used to send responses to the command, which will be sent to the
   connection from which it originated.
 * `&ClientServer` - will provide a reference to the `ClientServer` which is
   handling the command.
 * `&Network` - will provide a reference to the current network state.
 * `ServicesTarget` - will require that the network has an active services
   instance, and return an interface to send remote requests to it.

In addition, command source types can be used as ambient argument types:

 * `CommandSource` - describes the logical source of the command, whether a
   user or a pre-client.
 * `UserSource` - requires that the source must be a user that has completed
   registration, and provides the user information.
 * `PreClientSource` - requires that the source must be a pre-client connection
   that has not completed registration, and provides its information.
 * `LoggedInUserSource` - requires that the source is a user who is currently
   logged in to an account, and provides both the user and the account
   information.

If the requirements imposed by a source type are not met, an appropriate error
message will be presented to the source connection and the handler will not be
invoked.

### Positional argument types

Positional argument types are parsed from the arguments provided by the client,
in left-to-right order. Each handler argument will typically consume one
protocol argument, though the conditional types (see below) may or may not do so.

 * `&str` - the argument provided, verbatim.
 * `u32` - parse the argument into an unsigned integer.
 * `Nickname`, `ChannelKey`, `ChannelRoleName`, `CustomRoleName` - validate that
   the provided argument is valid for the relevant name type, and provide it.
 * `wrapper::{User,Channel,Account,ChannelRegistration}` - treat the provided
   argument as a nick or channel name and look up the relevant object.
 * `TargetParameter` - the provided argument can be either a nickname or a
   channel name (i.e., something that can be a message target); in either case
   look up the relevant network object and provide it.
 * `RegisteredChannel` - the argument is the name of a channel which must both
   exist and be registered; the `RegisteredChannel` type will contain references
   to both.

### Conditional argument wrappers

The three conditional argument wrappers all provide an argument if it is present
and valid for the relevant argument type, but differ in their behaviour if it
is not.

 * `Option<T>` - the argument will always be consumed if present. If it is
   present and can be parsed, it will be provided, otherwise `None` will be
   passed in its place.
 * `IfParses<T>` - if the next argument can be parsed as a T, then provide it,
   otherwise do not consume it and allow it to be parsed for the next positional
   argument. This allows, for example, an optional duration argument in a
   non-final position; if the argument is (for example) an integer it will be
   treated as a duration, if not then there is no duration and the next argument
   is processed.
 * `Conditional<T>` - attempt to parse a `T`, storing the error if it fails.
   Calling `require()?` on the `Conditional` will return the parameter and
   propagate the error to the user if it fails; this is used for parameters
   which may be required based on the values of earlier arguments.

`Conditional` can also be used for an ambient argument type, for example if
a command only requires a services instance for some branches and not others.



