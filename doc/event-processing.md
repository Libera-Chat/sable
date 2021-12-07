# Processing Events

All events have the same basic structure, with varying detail fields.
As well as the event ID, they contain the Unix timestamp when it was
created, a target object ID, and a vector clock.

The clock contains, for each server ID, the most recent event from that
server which had been processed by the server emitting the event, at
the time the event was created. This provides a (partial) dependency
ordering for events. An incoming event whose event clock is not
less than or equal to the server's current clock must be held in a
pending queue until its dependencies can be satisfied.

Most events are handled in a fairly obvious manner, creating, updating 
or deleting state objects. Those which require more complex logic, such
as for conflict resolution, are described below.

Note that events may be processed by a given server in any order that
is consistent with the dependency graph, and may be processed in 
different orders by different servers. Note also that once an event
has been accepted into a server's event log, it cannot be
retrospectively removed or reverted. Events must be processed in a
manner that ensures that any two servers with the same event clock
also have the same view of network state.

## BindNickname

    #[target_type(NicknameId)]
    struct BindNickname {
        pub user: UserId,
    }

The event's target ID is the nickname being bound; the `user` field is
the ID of the user binding the nick.

If no binding exists for the target nickname, then create it and remove
any existing binding for the user.

If a binding already exists for the target nickname, then conflict
resolution is required:

 * If the existing binding has a lower timestamp than the one being
   processed, or has an equal timestamp and a lower user ID, then the
   existing binding is left intact, and the user identified in the
   binding currently being processed is collided. The incoming binding
   is not processed.
 * Otherwise, the existing binding is removed, and the user identified
   in the now-removed binding is collided. The new binding is then
   processed as normal.

If a user must be collided, then:

 * A numeric nickname based on the FNV32A1 hash of the user's ID is
   generated.
 * If that nickname is not currently bound, then bind it to the user.
 * If that nickname is currently bound, then:
   * If the event which caused this collision depends upon the event
     which created the existing binding, then the user currently being
     collided is disconnected with an error.
   * If the event which caused this collision does not depend upon the
     event which created the existing binding, then both the user
     currently being collided and the user identified in the existing
     binding are disconnected with an error.
   * It is not possible for the existing binding's event ID to depend
     upon the event currently being processed.

For any user whose nickname binding changes, or which is disconnected
as a result of this process, emit appropriate change events for
clients to be notified.


## NewUser

    #[target_type(UserId)]
    struct NewUser {
        pub nickname: Nickname,
        pub username: Username,
        pub visible_hostname: Hostname,
        pub realname: String,
        pub mode_id: UModeId,
        pub server: ServerId,
    }

Although the User object does not contain a nickname, as nickname
ownership is defined by the separate NickBinding object, the NewUser
event contains the intended nickname. As part of processing, an
appropriate nickname binding is created, applying all of the conflict
resolution rules listed above.

If these rules result in the new user being collided, then a nickname
change event must be emitted for that user, notifying a change from
the nickname listed in the `NewUser` event to the collided nickname.


## UserQuit

    #[target_type(UserId)]
    struct UserQuit {
        pub message: String,
    }

Remove the target user from the network state, including all associated
user mode and membership objects.

## BindChannelName



## ServerQuit

    #[target_type(ServerId)]
    struct ServerQuit {
        pub introduced_by: EventId,
    }

Remove the target server from the network state, including all users
currently connected to that server, following the rules above for each
of those users.
