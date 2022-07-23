//! An inter-process channel using Unix datagram sockets

use serde::{
    Serialize,
    de::DeserializeOwned,
};
use tokio::{
    net::UnixDatagram,
};
use std::{
    net::Shutdown,
    marker::PhantomData,
    os::unix::io::{
        RawFd,
        FromRawFd,
        IntoRawFd,
    },
};
use parking_lot::Mutex;
use thiserror::Error;

use bincode::{
    Options,
    DefaultOptions,
};

#[derive(Debug,Error)]
pub enum Error
{
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialisation error: {0}")]
    Serialize(#[from] bincode::Error)
}

impl From<Error> for std::io::Error
{
    fn from(e: Error) -> Self
    {
        match e {
            Error::Io(e) => e,
            Error::Serialize(e) => std::io::Error::new(std::io::ErrorKind::Other, e)
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn channel<T: Serialize + DeserializeOwned>(max_size: u64) -> Result<(Sender<T>, Receiver<T>)>
{
    let (send_sock, recv_sock) = UnixDatagram::pair()?;

    Ok((Sender::new(send_sock, max_size), Receiver::new(recv_sock, max_size)))
}

pub struct Sender<T: Serialize>
{
    // Option is purely so we can move out of this while implementing Drop
    socket: Option<UnixDatagram>,
    max_len: u64,
    _phantom: PhantomData<T>
}

impl <T: Serialize> Sender<T>
{
    fn new(socket: UnixDatagram, max_len: u64) -> Self
    {
        Self { socket: Some(socket), max_len, _phantom: PhantomData }
    }

    pub async fn send(&self, data: &T) -> Result<()>
    {
        let bytes = DefaultOptions::new().with_limit(self.max_len).serialize(data)?;
        self.socket.as_ref().unwrap().send(&bytes).await?;

        Ok(())
    }

    /// Construct a `Sender` which takes ownership of the given raw FD.
    ///
    /// # Safety
    ///
    /// The provided FD must not be used by anything else after being passed to this function,
    /// and must have been obtained by calling `into_raw_fd` on a `Sender` of the same type.
    pub unsafe fn from_raw_fd(fd: RawFd, max_size: u64) -> std::io::Result<Self>
    {
        let std_socket = std::os::unix::net::UnixDatagram::from_raw_fd(fd);
        Ok(Self::new(UnixDatagram::from_std(std_socket)?, max_size))
    }

    /// Consume a `Sender` and return the underlying file descriptor
    ///
    /// # Safety
    ///
    /// Using the returned FD for anything other than `Self::from_raw_fd` may cause unpredictable
    /// behaviour in the corresponding `Receiver`.
    pub unsafe fn into_raw_fd(mut self) -> std::io::Result<RawFd>
    {
        let std_socket = self.socket.take().unwrap().into_std()?;
        Ok(std_socket.into_raw_fd())
    }
}

impl<T: Serialize> Drop for Sender<T>
{
    fn drop(&mut self)
    {
        if let Some(socket) = self.socket.take()
        {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }
}

pub struct Receiver<T: DeserializeOwned>
{
    // Option is purely so we can move out of this while implementing Drop
    socket: Option<UnixDatagram>,
    max_len: u64,
    recv_buffer: Mutex<Vec<u8>>,
    _phantom: PhantomData<T>
}

impl<T: DeserializeOwned> Receiver<T>
{
    fn new(socket: UnixDatagram, max_len: u64) -> Self
    {
        let mut recv_buf = Vec::new();
        recv_buf.resize(max_len as usize, 0u8);

        Self { socket: Some(socket), max_len, recv_buffer: Mutex::new(recv_buf), _phantom: PhantomData }
    }

    pub async fn recv(&self) -> Result<T>
    {
        let sock = self.socket.as_ref().unwrap();

        loop {
            sock.readable().await?;

            let mut buffer = self.recv_buffer.lock();

            if let Ok(recv_len) = sock.try_recv(&mut buffer)
            {
                break Ok(DefaultOptions::new().with_limit(self.max_len).deserialize(&buffer[..recv_len])?);

            }
        }
    }

    /// Construct a `Receiver` which takes ownership of the given raw FD.
    ///
    /// # Safety
    ///
    /// The provided FD must not be used by anything else after being passed to this function,
    /// and must have been obtained by calling `into_raw_fd` on a `Receiver` of the same type.
    pub unsafe fn from_raw_fd(fd: RawFd, max_size: u64) -> std::io::Result<Self>
    {
        let std_socket = std::os::unix::net::UnixDatagram::from_raw_fd(fd);
        Ok(Self::new(UnixDatagram::from_std(std_socket)?, max_size))
    }

    /// Consume a `Receiver` and return the underlying file descriptor
    ///
    /// # Safety
    ///
    /// Using the returned FD for anything other than `Self::from_raw_fd` may cause unpredictable
    /// behaviour in the corresponding `Sender`.
    pub unsafe fn into_raw_fd(mut self) -> std::io::Result<RawFd>
    {
        let std_socket = self.socket.take().unwrap().into_std()?;
        Ok(std_socket.into_raw_fd())
    }
}

impl<T: DeserializeOwned> Drop for Receiver<T>
{
    fn drop(&mut self)
    {
        if let Some(socket) = self.socket.take()
        {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }
}
