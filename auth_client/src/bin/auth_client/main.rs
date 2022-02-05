use auth_client::*;

use std::env;
use std::os::unix::io::FromRawFd;

use tokio::{
    sync::mpsc::{
        unbounded_channel,
    },
    select
};
use tokio_unix_ipc::{
    Sender as IpcSender,
    Receiver as IpcReceiver,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let mut args = env::args();
    args.next();

    let control_fd = args.next().unwrap();
    let event_fd = args.next().unwrap();

    let control_fd: i32 = control_fd.trim().parse()?;
    let event_fd: i32 = event_fd.trim().parse()?;

    let (control_recv, event_send) = unsafe
    {
        (IpcReceiver::<ControlMessage>::from_raw_fd(control_fd), IpcSender::<AuthEvent>::from_raw_fd(event_fd))
    };

    let (dns_event_send, mut dns_event_recv) = unbounded_channel();

    let client = dns_client::InternalDnsClient::new(dns_event_send);

    loop
    {
        select!(
            control = control_recv.recv() =>
            {
                match control?
                {
                    ControlMessage::Shutdown =>
                    {
                        break;
                    }
                    ControlMessage::StartDnsLookup(conn_id, addr) =>
                    {
                        client.start_lookup(conn_id, addr);
                    }
                }
            },
            event = dns_event_recv.recv() =>
            {
                match event
                {
                    Some(evt) =>
                    {
                        event_send.send(AuthEvent::DnsResult(evt)).await?;
                    }
                    None =>
                    {
                        break;
                    }
                }
            }
        );
    }

    Ok(())
}

mod dns_client;