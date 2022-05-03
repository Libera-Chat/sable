use client_listener::*;

use std::env;
use std::os::unix::io::FromRawFd;

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
        (IpcReceiver::<ControlMessage>::from_raw_fd(control_fd), IpcSender::<InternalConnectionEvent>::from_raw_fd(event_fd))
    };

    let mut process = ListenerProcess::new(control_recv, event_send);
    process.run().await.expect("Error in listener process");
    Ok(())
}