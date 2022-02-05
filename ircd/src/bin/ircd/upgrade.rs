use ircd::*;
use irc_server::server::*;

use std::os::unix::{
    io::{
        RawFd,
        IntoRawFd,
        FromRawFd,
    },
    process::CommandExt,
};
use std::{
    io::Seek,
    process::Command
};

use memfd::*;

#[derive(Serialize,Deserialize)]
pub struct ApplicationState
{
    pub server_state: ServerState,
    pub listener_state: client_listener::SavedListenerCollection,
    pub sync_state: EventLogState,
}

pub fn read_upgrade_state(fd: RawFd) -> ApplicationState
{
    let memfd = unsafe { Memfd::from_raw_fd(fd) };
    let file = memfd.as_file();

    serde_json::from_reader(file).expect("Failed to unpack upgrade state")
}

fn prepare_upgrade(state: ApplicationState) -> RawFd
{
    let memfd = MemfdOptions::default().close_on_exec(false).create("upgrade_state").expect("Failed to create upgrade memfd");
    let mut file = memfd.as_file();

    serde_json::to_writer(file, &state).expect("Failed to serialise server state");
    file.rewind().expect("Failed to rewind memfd");
    memfd.into_raw_fd()
}

pub(super) fn exec_upgrade(exe: &Path, opts: super::Opts, state: ApplicationState) -> !
{
    let fd = prepare_upgrade(state);
    let args = ["--server-conf",
                opts.server_conf.to_str().unwrap(),
                "--network-conf",
                opts.network_conf.to_str().unwrap(),
                "--upgrade-state-fd",
                &fd.to_string()];

    log::debug!("Executing upgrade: {:?} {:?}", exe, args);

    let err = Command::new(exe)
                      .args(args)
                      .exec();

    panic!("exec() failed on upgrade: {}", err);
}