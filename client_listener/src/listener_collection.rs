use crate::*;
use crate::internal::*;

use std::{
    collections::HashMap,
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
    task::JoinHandle,
};
use tokio_unix_ipc::{
    Sender as IpcSender,
    Receiver as IpcReceiver,
    channel as ipc_channel
};

use std::os::unix::{
    io::{
        RawFd,
        IntoRawFd,
        FromRawFd
    },
    process::CommandExt
};
use std::process::{
    Command,
    Child
};

#[derive(serde::Serialize,serde::Deserialize)]
pub struct SavedListenerCollection
{
    control_sender: RawFd,
    event_receiver: RawFd,
    id_gen: ListenerIdGenerator,
    connection_data: HashMap<ConnectionId, ConnectionData>,
}

type CommResult = std::io::Result<(IpcSender<ControlMessage>, IpcReceiver<InternalConnectionEvent>)>;

pub struct ListenerCollection
{
    listener_id_generator: ListenerIdGenerator,
    control_sender: UnboundedSender<ControlMessage>,
    comm_task: JoinHandle<CommResult>,
    connection_data: HashMap<ConnectionId, ConnectionData>,
    // We can't reconstruct the Child after save/resume, so we have to do without then
    child_process: Option<Child>,
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

        let comm_task = task::spawn(run_communication_task(control_send, local_control_send.clone(),
                                            local_control_recv, event_recv, event_channel));

        let ret = Self {
            listener_id_generator: ListenerIdGenerator::new(0),
            control_sender: local_control_send,
            comm_task: comm_task,
            connection_data: HashMap::new(),
            child_process: Some(child)
        };

        Ok(ret)
    }

    pub async fn save(self) -> std::io::Result<SavedListenerCollection>
    {
        log::debug!("Saving state");
        log::debug!("Stopping control task...");
        self.control_sender.send(ControlMessage::SaveForUpgrade).map_err(|_| std::io::Error::from(std::io::ErrorKind::Other))?;
        let (ctl_send, evt_recv) = self.comm_task.await??;
        log::debug!("control task done");

        let (ctl_fd, evt_fd) = unsafe
        {
            let control_fd = ctl_send.into_raw_fd();
            let event_fd = evt_recv.into_raw_fd();

            use libc::{fcntl, F_GETFD, F_SETFD, FD_CLOEXEC};

            let cfd_flags = fcntl(control_fd, F_GETFD);
            fcntl(control_fd, F_SETFD, cfd_flags & !FD_CLOEXEC);
            let efd_flags = fcntl(event_fd, F_GETFD);
            fcntl(event_fd, F_SETFD, efd_flags & !FD_CLOEXEC);

            (control_fd, event_fd)
        };

        log::debug!("unwrapped fds");

        Ok(SavedListenerCollection {
            control_sender: ctl_fd,
            event_receiver: evt_fd,
            id_gen: self.listener_id_generator,
            connection_data: self.connection_data
        })
    }

    pub fn resume(state: SavedListenerCollection, event_channel: Sender<ConnectionEvent>) -> std::io::Result<Self>
    {
        let (control_sender, event_receiver) = unsafe
        {
            (IpcSender::<ControlMessage>::from_raw_fd(state.control_sender),
             IpcReceiver::<InternalConnectionEvent>::from_raw_fd(state.event_receiver))
        };

        let (local_control_send, local_control_recv) = unbounded_channel();

        let handle = tokio::spawn(run_communication_task(control_sender, local_control_send.clone(), local_control_recv, event_receiver, event_channel));

        Ok(Self {
            control_sender: local_control_send,
            comm_task: handle,
            listener_id_generator: state.id_gen,
            connection_data: state.connection_data,
            child_process: None
        })
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

    pub fn restore_connection(&self, data: ConnectionData) -> Connection
    {
        Connection::new(data.id, data.conn_type, data.remote_addr, self.control_sender.clone())
    }

    pub async fn shutdown(self)
    {
        let _ = self.control_sender.send(ControlMessage::Shutdown);
        let _ = self.comm_task.await;
        if let Some(mut child) = self.child_process
        {
            let _ = child.wait();
        }
    }
}

async fn run_communication_task<'a>(
        control_send: IpcSender<ControlMessage>,
        local_control_send: UnboundedSender<ControlMessage>,
        mut local_control_recv: UnboundedReceiver<ControlMessage>,
        event_receiver: IpcReceiver<InternalConnectionEvent>,
        event_sender: Sender<ConnectionEvent>,
    ) -> CommResult
{
    loop
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
                            let new_connection = Connection::new(data.id, data.conn_type, data.remote_addr, local_control_send.clone());
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
                            control_send.send(control).await?;
                            break;
                        }
                        else if matches!(control, ControlMessage::SaveForUpgrade)
                        {
                            break;
                        }
                        control_send.send(control).await?;
                    }
                    None => break
                }
            }
        }
    }
    Ok((control_send, event_receiver))
}
