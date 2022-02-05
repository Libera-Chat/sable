use client_listener::*;

use std::{
    collections::HashMap,
    net::SocketAddr,
    env::current_exe
};

use tokio::{
//    select,
    sync::mpsc::{
        channel
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    tracing_subscriber::fmt::init();

    let mut connections = HashMap::new();
    let (event_send, mut event_recv) = channel(128);

    let addr: SocketAddr = "127.0.0.1:5555".parse()?;
    let exe = current_exe()?.parent().unwrap().parent().unwrap().join("listener_process");
    tracing::info!("exe = {:?}", exe);

    let listeners = ListenerCollection::with_exe_path(exe, event_send)?;
    let _id = listeners.add_listener(addr, ConnectionType::Clear)?;

    while let Some(event) = event_recv.recv().await
    {
        match event.detail
        {
            ConnectionEventDetail::NewConnection(conn) =>
            {
                tracing::info!("New connection {:?}", event.source);
                connections.insert(event.source, conn);
            }
            ConnectionEventDetail::Message(msg) =>
            {
                tracing::info!("Message from {:?}: {}", event.source, msg);
                if let Some(conn) = connections.get(&event.source)
                {
                    conn.send(format!("{}\n", msg));
                }
                else
                {
                    tracing::warn!("Got message from unknown connection id {:?}", event.source);
                }
            }
            ConnectionEventDetail::Error(err) =>
            {
                tracing::error!("Got error {}", err);
            }
        }
    }

    Ok(())
}