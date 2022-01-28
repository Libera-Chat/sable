use crate::*;
use crate::internal::*;

use std::{
    net::SocketAddr,
    path::Path,
    env::current_exe,
    io,
};

use tokio::{
    sync::mpsc::{
        Sender,
        UnboundedSender,
        UnboundedReceiver,
        unbounded_channel
    },
    select,
    task,
};
use tokio_unix_ipc::{
    Sender as IpcSender,
    Receiver as IpcReceiver,
    channel as ipc_channel
};

use std::os::unix::{
    io::IntoRawFd,
    process::CommandExt
};
use std::process::{
    Command,
    Child
};

pub struct ListenerCollection
{
    listener_id_generator: ListenerIdGenerator,
    control_sender: UnboundedSender<ControlMessage>,
    child_process: Child,
}

impl ListenerCollection
{
    pub fn new(event_channel: Sender<ConnectionEvent>) -> std::io::Result<Self>
    {
        let my_path = current_exe()?;
        let dir = my_path.parent().ok_or(io::ErrorKind::NotFound)?;
        let default_listener_path = dir.join("listener_process");

        Self::with_exe_path(default_listener_path, event_channel)
    }

    pub fn with_exe_path(exec_path: impl AsRef<Path>, event_channel: Sender<ConnectionEvent>) -> std::io::Result<Self>
    {
        let (control_send, control_recv) = ipc_channel()?;
        let (event_send, event_recv) = ipc_channel()?;
        let (local_control_send, local_control_recv) = unbounded_channel();

        let child = unsafe
        {
            let control_fd = control_recv.into_raw_fd();
            let event_fd = event_send.into_raw_fd();

            Command::new(exec_path.as_ref())
                    .args([control_fd.to_string(), event_fd.to_string()])
                    .pre_exec(move || {
                        use libc::{fcntl, F_GETFD, F_SETFD, FD_CLOEXEC};

                        let cfd_flags = fcntl(control_fd, F_GETFD);
                        fcntl(control_fd, F_SETFD, cfd_flags & !FD_CLOEXEC);
                        let efd_flags = fcntl(event_fd, F_GETFD);
                        fcntl(event_fd, F_SETFD, efd_flags & !FD_CLOEXEC);
                        Ok(())
                    })
                    .spawn()?
        };

        let ret = Self {
            listener_id_generator: ListenerIdGenerator::new(0),
            control_sender: local_control_send.clone(),
            child_process: child
        };

        task::spawn(run_communication_task(control_send, local_control_send, local_control_recv, event_recv, event_channel));

        Ok(ret)
    }

    pub fn add_listener(&self, address: SocketAddr, conn_type: ConnectionType) -> Result<ListenerId,ListenerError>
    {
        let id = self.listener_id_generator.next();

        let message = ControlMessage::Listener(id, ListenerControlDetail::Add(address, conn_type));
        self.control_sender.send(message)?;
        return Ok(id)
    }

    pub fn load_tls_certificates(&self, settings: TlsSettings) -> Result<(), ListenerError>
    {
        Ok(self.control_sender.send(ControlMessage::LoadTlsSettings(settings))?)
    }

    fn shutdown(&mut self)
    {
        let _ = self.control_sender.send(ControlMessage::Shutdown);
        let _ = self.child_process.wait();
    }
}

impl Drop for ListenerCollection
{
    fn drop(&mut self)
    {
        self.shutdown()
    }
}

async fn run_communication_task<'a>(
        control_send: IpcSender<ControlMessage>,
        local_control_send: UnboundedSender<ControlMessage>,
        mut local_control_recv: UnboundedReceiver<ControlMessage>,
        event_receiver: IpcReceiver<InternalConnectionEvent>,
        event_sender: Sender<ConnectionEvent>,
    ) -> std::io::Result<()>
{
    let mut done = false;

    while !done
    {
        select! {
            event = event_receiver.recv() =>
            {
                if let Ok(evt) = event
                {
                    use InternalConnectionEvent::*;
                    let translated_event = match evt
                    {
                        NewConnection(data) => {
                            let new_connection = Connection::new(data.id, data.conn_type, data.endpoint, local_control_send.clone());
                            ConnectionEvent::new(new_connection.id, new_connection)
                        },
                        ConnectionError(id, err) => ConnectionEvent::error(id, err),
                        Message(id, msg) => ConnectionEvent::message(id, msg),
                        _ => continue
                    };
                    if let Err(e) = event_sender.send(translated_event).await {
                        log::error!("Error sending connection event: {}", e);
                    }
                }
            },
            control = local_control_recv.recv() =>
            {
                match control
                {
                    Some(control) => {
                        if matches!(control, ControlMessage::Shutdown)
                        {
                            done = true;
                        }
                        control_send.send(control).await?;
                    }
                    None => break
                }
            }
        }
    }
    Ok(())
}
