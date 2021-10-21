pub mod ircd;
pub mod utils;

use ircd::*;
use event::*;

use async_std::{
    task,
    channel,
    prelude::*,
};
use log;
use simple_logger::SimpleLogger;
use gossip::*;

struct UpdateHandler
{
    send_channel: channel::Sender<Event>,
}

impl gossip::UpdateHandler for UpdateHandler
{
    fn on_update(&self, update: gossip::Update)
    {
        if let Ok(event) = serde_json::from_slice::<Event>(update.content())
        {
            // Panic if we can't send the event for processing
            self.send_channel.try_send(event).unwrap();
        }
    }
}

async fn run_push_task(channel: channel::Receiver<Event>, service: gossip::GossipService<UpdateHandler>)
{
    let mut channel = channel;
    while let Some(event) = channel.next().await
    {
        service.submit(serde_json::to_vec(&event).unwrap()).unwrap();
    }
}

fn main()
{
    let args: Vec<String> = std::env::args().collect();

    let server_id: i64 = args[1].parse().unwrap();
    let server_name = &args[2];
    let address = &args[3];

    let listen_addr = format!("{}:6667", address);
    let gossip_addr = format!("{}:6668", address);
    let peer = args.get(4).map(|s| format!("{}:6668", s));

    let peer_init = || { peer.map(|x| vec!(Peer::new(x))) };

    SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();

    task::block_on(async {
        let (evt_send, evt_recv) = channel::unbounded::<Event>();

        let mut server = irc::Server::new(server_id, server_name.clone(), evt_send);
        println!("addr: {}", listen_addr);
        server.add_listener(listen_addr.parse().unwrap());

        let gossip_handler = Box::new(UpdateHandler { send_channel: server.get_event_sender() });
        let mut gossip_service: GossipService<UpdateHandler> = 
                GossipService::new(gossip_addr.parse().unwrap(), PeerSamplingConfig::default(), GossipConfig::default());

        gossip_service.start(Box::new(peer_init), gossip_handler).unwrap();

        task::spawn(run_push_task(evt_recv, gossip_service));

        server.run().await;
    });
}