use client_listener::*;

use std::{
    collections::HashMap,
    net::SocketAddr,
    error::Error,
    os::unix::io::{
        IntoRawFd,
    },
    os::unix::process::CommandExt,
    env::current_exe,
    process::Command,
    io::{
        Read,
        Write,
        Seek,
    },
    convert::TryInto
};

use memfd::*;

use tokio::{
//    select,
    sync::mpsc::{
        channel
    },
};

#[derive(serde::Serialize,serde::Deserialize)]
struct SaveData
{
    listeners: SavedListenerCollection,
    connections: Vec<ConnectionData>
}

fn to_memfd(data: SaveData) -> Result<i32, Box<dyn Error>>
{
    let memfd = MemfdOptions::default().close_on_exec(false).create("listener_data")?;
    let mut memfile = memfd.as_file();
    let data = serde_json::to_vec(&data)?;

    tracing::debug!("serialised data: ({}) {:?}", data.len(), data);

    memfile.set_len(data.len().try_into()?)?;
    memfile.write_all(&data)?;

    // Since we're passing the open fd across the exec, we need to rewind it explicitly
    // as it's not being reopened
    memfile.rewind()?;

    let fd = memfd.into_raw_fd();

    tracing::debug!("wrote data to fd {:?}", fd);

    Ok(fd)
}

fn from_memfd(fd: i32) -> Result<SaveData, Box<dyn Error>>
{
    let memfd = Memfd::try_from_fd(fd).unwrap();
    let mut memfile = memfd.as_file();

    let mut data = Vec::new();
    memfile.read_to_end(&mut data)?;

    tracing::debug!("Read data: {:?}", data);
    Ok(serde_json::from_slice(&data)?)
}

async fn do_restart(listeners: ListenerCollection, connections: HashMap<ConnectionId, Connection>) -> !
{
    let data = SaveData {
        listeners: listeners.save().await.unwrap(),
        connections: connections.into_iter().map(|(_,c)| c.save()).collect()
    };
    let fd = to_memfd(data).unwrap();

    tracing::debug!("executing restart");
    Command::new(current_exe().unwrap())
                    .args([fd.to_string()])
                    .exec();

    panic!("Couldn't exec?");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    tracing_subscriber::fmt::init();

    let (event_send, mut event_recv) = channel(128);

    let args: Vec<_> = std::env::args().collect();

    let mut connections = HashMap::new();

    let listeners = if let Some(fd) = args.get(1)
    {
        tracing::info!("Resuming from FD {}", fd);
        let data = from_memfd(fd.parse()?)?;

        tracing::debug!("Reloading listeners");
        let listeners = ListenerCollection::resume(data.listeners, event_send)?;

        for conndata in data.connections
        {
            let conn = listeners.restore_connection(conndata);
            connections.insert(conn.id, conn);
        }

        listeners
    }
    else
    {
        tracing::info!("No FD supplied; starting from cold");
        let addr: SocketAddr = "127.0.0.1:5555".parse()?;
        let exe = current_exe()?.parent().unwrap().parent().unwrap().join("listener_process");

        let listeners = ListenerCollection::with_exe_path(exe, event_send)?;
        let _id = listeners.add_listener(addr, ConnectionType::Clear)?;
        listeners
    };

    tracing::debug!("Starting event loop");

    while let Some(event) = event_recv.recv().await
    {
        tracing::debug!("Got event");
        match event.detail
        {
            ConnectionEventDetail::NewConnection(conn) =>
            {
                tracing::info!("New connection {:?}", event.source);
                connections.insert(event.source, conn);
            }
            ConnectionEventDetail::Message(msg) =>
            {
                if msg == "restart"
                {
                    tracing::info!("Restarting...");
                    do_restart(listeners, connections).await;
                }

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