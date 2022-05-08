use client_listener::*;

use std::env;

use sable_ipc::{
    Sender as IpcSender,
    Receiver as IpcReceiver,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    console_subscriber::init();

    let mut args = env::args();
    args.next();

    let control_fd = args.next().unwrap();
    let event_fd = args.next().unwrap();

    let control_fd: i32 = control_fd.trim().parse()?;
    let event_fd: i32 = event_fd.trim().parse()?;

    let (control_recv, event_send) = unsafe
    {
        (IpcReceiver::<ControlMessage>::from_raw_fd(control_fd, client_listener::MAX_CONTROL_SIZE).expect("Failed to unpack control receiver"),
         IpcSender::<InternalConnectionEvent>::from_raw_fd(event_fd, client_listener::MAX_MSG_SIZE).expect("Failed to unpack event sender"))
    };

    let mut process = ListenerProcess::new(control_recv, event_send);
    process.run().await.expect("Error in listener process");
    tracing::warn!("listener shutting down");
    Ok(())
}