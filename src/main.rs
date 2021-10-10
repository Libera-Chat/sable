pub mod ircd;
pub mod utils;

use ircd::*;
use async_std::{
    task,
};
use log;
use simple_logger::SimpleLogger;

static SERVER_ID: i64 = 1;

fn main()
{
    let server_name = String::from("test.server");

    SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();

    task::block_on(async {
        let mut server = irc::Server::new(SERVER_ID, server_name);
        server.add_listener("127.0.0.1:6667".parse().unwrap());
        server.run().await;
    });
}