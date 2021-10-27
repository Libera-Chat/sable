pub mod ircd;
mod utils;
mod network_sync;

use ircd::*;
use event::*;
use network_sync::NetworkSync;

use async_std::{
    task,
    channel,
    prelude::*,
};
use futures::{
    FutureExt,
    select,
};
use log;
use simple_logger::SimpleLogger;

fn main()
{
    let args: Vec<String> = std::env::args().collect();

    let server_id = ServerId::new(args[1].parse().unwrap());
    let server_name = &args[2];
    let address = &args[3];

    let listen_addr = format!("{}:6667", address);
    let gossip_addr = format!("{}:6668", address);
    let peer_addr = args.get(4).map(|s| format!("{}:6668", s));

    SimpleLogger::new().with_level(log::LevelFilter::Debug)
                       .with_module_level("gossip", log::LevelFilter::Error)
                       .init().unwrap();

    // Communication channels:
    //
    // outbound: events from the log to the network
    // inbound: events from the network to the log
    // log: events from the log to (server, network)
    // server: events to server from log
    // create: event details from the server to be created by the log and rebroadcast
    let (outbound_send, outbound_recv) = channel::unbounded::<Event>();
    let (inbound_send, mut inbound_recv) = channel::unbounded::<Event>();
    let (log_send, mut log_recv) = channel::unbounded::<Event>();
    let (server_send, server_recv) = channel::unbounded::<Event>();
    let (new_send, mut new_recv) = channel::unbounded::<(ObjectId,EventDetails)>();

    let event_id_gen = EventIdGenerator::new(server_id, 1);
    let mut event_log = EventLog::new(event_id_gen, Some(log_send));

    task::spawn(async move {
        // This task owns the event log, and pumps events:
        //
        // - from new_recv (the Server producing state changes) into the log
        // - from inbound_recv (events appearing from the network sync) into the log
        // - from the log to the server for processing
        // - from the log to the network for outbound sync
        loop
        {
            select!
            {
                evt = inbound_recv.next().fuse() => {
                    match evt {
                        Some(evt) => event_log.add(evt),
                        None => break
                    }
                },
                evt = new_recv.next().fuse() => {
                    match evt {
                        Some((id, detail)) => {
                            let event = event_log.create(id, detail);
                            event_log.add(event.clone());
                            if let Err(_) = outbound_send.send(event).await
                            {
                                break;
                            }
                        }
                        None => break
                    }
                },
                evt = log_recv.next().fuse() => {
                    match evt {
                        Some(evt) => {
                            if let Err(_) = server_send.send(evt).await
                            {
                                break;
                            }
                        },
                        None => break
                    }
                }
            }
        }
    });

    let mut server = irc::Server::new(server_id,
                                        server_name.clone(),
                                        server_recv,
                                        new_send);
    println!("addr: {}", listen_addr);
    server.add_listener(listen_addr.parse().unwrap());

    NetworkSync::start(gossip_addr, peer_addr, inbound_send, outbound_recv);

    task::block_on(async {
        server.run().await;
    });
}