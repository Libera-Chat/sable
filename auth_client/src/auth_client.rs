use crate::*;
use client_listener::ConnectionId;

use tokio::{
    sync::mpsc::{
        Sender,
        UnboundedSender,
        UnboundedReceiver,
        unbounded_channel,
    },
    sync::oneshot,
    task,
    select
};
use tokio_unix_ipc::{
    Sender as IpcSender,
    Receiver as IpcReceiver,
    channel as ipc_channel
};
use std::{
    os::unix::{
        io::{
            RawFd,
            IntoRawFd,
            FromRawFd,
        },
        process::CommandExt,
    },
    path::Path,
    env::current_exe,
    io,
    process::{
        Command,
        Child
    },
    net::IpAddr,
};

/// Library interface to the auth client process.
pub struct AuthClient
{
    control_sender: UnboundedSender<ControlMessage>,
    comm_task_shutdown: oneshot::Sender<()>,
    comm_task: task::JoinHandle<CommResult>,
    child_process: Option<Child>
}

/// Opaque saved-state to reconstitute an AuthClient after an upgrade
#[derive(serde::Serialize,serde::Deserialize)]
pub struct AuthClientState
{
    control_fd: RawFd,
    event_fd: RawFd,
}

type CommResult = io::Result<(IpcSender<ControlMessage>, IpcReceiver<AuthEvent>)>;

#[tracing::instrument(skip_all)]
async fn run_communication_task(
        control_sender: IpcSender<ControlMessage>,
        event_receiver: IpcReceiver<AuthEvent>,
        mut control_receiver: UnboundedReceiver<ControlMessage>,
        event_sender: Sender<AuthEvent>,
        mut shutdown_receiver: oneshot::Receiver<()>
    ) -> CommResult
{
    loop
    {
        select!(
            event = event_receiver.recv() =>
            {
                if let Err(e) = event_sender.send(event?).await {
                    tracing::error!("Error sending connection event: {}", e);
                }
        },
            control = control_receiver.recv() =>
            {
                match control
                {
                    Some(ctl) => {
                        let is_shutdown = matches!(ctl, ControlMessage::Shutdown);
                        control_sender.send(ctl).await?;

                        if is_shutdown
                        {
                            break;
                        }
                    }
                    None => {
                        control_sender.send(ControlMessage::Shutdown).await?;
                        break;
                    }
                };
            },
            _ = &mut shutdown_receiver =>
            {
                break;
            }
        );
    }

    Ok((control_sender, event_receiver))
}

impl AuthClient
{
    pub fn new(event_channel: Sender<AuthEvent>) -> std::io::Result<Self>
    {
        let my_path = current_exe()?;
        let dir = my_path.parent().ok_or(io::ErrorKind::NotFound)?;
        let default_listener_path = dir.join("auth_client");

        Self::with_exe_path(default_listener_path, event_channel)
    }

    pub fn with_exe_path(exec_path: impl AsRef<Path>, event_channel: Sender<AuthEvent>) -> std::io::Result<Self>
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

        let (shutdown_send, shutdown_recv) = oneshot::channel();

        let comm_task = task::spawn(run_communication_task(control_send, event_recv,
                                            local_control_recv, event_channel, shutdown_recv));

        let ret = Self {
            control_sender: local_control_send,
            comm_task_shutdown: shutdown_send,
            comm_task: comm_task,
            child_process: Some(child)
        };

        Ok(ret)
    }

    #[tracing::instrument(skip(self))]
    pub fn start_dns_lookup(&self, conn_id: ConnectionId, addr: IpAddr)
    {
        self.control_sender.send(ControlMessage::StartDnsLookup(conn_id, addr)).ok();
    }

    #[tracing::instrument(skip(self))]
    pub async fn shutdown(self) -> io::Result<()>
    {
        self.control_sender.send(ControlMessage::Shutdown).map_err(|_| io::ErrorKind::Other)?;
        self.comm_task.await??;
        if let Some(mut child) = self.child_process
        {
            child.wait()?;
        }

        Ok(())
    }

    pub async fn save_state(self) -> std::io::Result<AuthClientState>
    {
        if self.comm_task_shutdown.send(()).is_err()
        {
            return Err(io::ErrorKind::Other.into());
        }
        let (control_send, event_recv) = self.comm_task.await??;

        let control_fd = control_send.into_raw_fd();
        let event_fd = event_recv.into_raw_fd();

        unsafe
        {
            use libc::{fcntl, F_GETFD, F_SETFD, FD_CLOEXEC};

            let cfd_flags = fcntl(control_fd, F_GETFD);
            fcntl(control_fd, F_SETFD, cfd_flags & !FD_CLOEXEC);
            let efd_flags = fcntl(event_fd, F_GETFD);
            fcntl(event_fd, F_SETFD, efd_flags & !FD_CLOEXEC);
        }

        Ok(AuthClientState {
            control_fd: control_fd,
            event_fd: event_fd
        })
    }

    pub fn resume(state: AuthClientState, event_channel: Sender<AuthEvent>) -> std::io::Result<Self>
    {
        let (control_send, event_recv) = unsafe
        {
            (IpcSender::<ControlMessage>::from_raw_fd(state.control_fd),
             IpcReceiver::<AuthEvent>::from_raw_fd(state.event_fd))
        };

        let (local_control_send, local_control_recv) = unbounded_channel();
        let (shutdown_send, shutdown_recv) = oneshot::channel();

        let comm_task = task::spawn(run_communication_task(control_send, event_recv,
            local_control_recv, event_channel, shutdown_recv));


        Ok(Self {
            control_sender: local_control_send,
            comm_task_shutdown: shutdown_send,
            comm_task: comm_task,
            child_process: None
        })
    }
}