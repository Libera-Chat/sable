# Handling Netsplits

Netsplits, as they affect current IRC networks, should be exceptionally rare
in the gossip-based mesh network used by sable. Nonetheless, it is important to
ensure that they are handled in a way that, if not an ideal user experience, is
internally consistent and avoids any possibility of desynchronisation.

Merging two divergent network states back together is a complex operation at
the best of times, and made more difficult by the requirement that it be
resolved simultaneously and independently on all servers, with all servers
reaching exactly the same state. The solution adopted here is not to attempt
it.

When a server is seen to leave the network, whether because it shut down or
due to network connectivity failure, all servers that process the server quit
event will keep a record of the (server id, epoch) pair which has quit. No
future incoming sync messages from that source pair will be accepted, and that
peer will no longer be eligible to send outgoing sync events.

When a server attempts to join the network, it will only be accepted if its
epoch is different from any previously seen epoch for that server ID. If that
is the case, then servers that process the new server event will enable that
peer for outgoing sync events.

The effect of this is that once the network is split into two or more subsets
which cannot communicate with each other, they will not attempt to relink. One
set must be manually chosen as the 'true' state of the network, and all servers
which have become detached from it must be restarted. With a new epoch ID, they
will be able to relink to the network and adopt that version of the network
state.

User experience may be improved under this scheme by servers automatically
recognising that they have become detached from the network, likely via a
quorum scheme, and automatically shutting down or degrading service in some
way. However, with the above measures in place this is not strictly necessary
to ensure network consistency.