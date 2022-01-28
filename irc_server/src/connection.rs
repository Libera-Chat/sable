use irc_network::*;
use crate::utils::*;
use crate::errors::*;

use tokio_rustls::TlsAcceptor;
use log::info;

use std::net::IpAddr;
use std::sync::Arc;


impl Drop for Connection
{
    fn drop(&mut self)
    {
        info!("Dropping connection {:?}", self.id);
    }
}
