use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;
use std::{io, sync::mpsc};
use std::process::{self, Stdio, ChildStderr, ChildStdin, ChildStdout};

use serenity::model::prelude::Message;
use serenity::prelude::Context;

use crate::mcserver_handler::McsrvHandler;
use crate::useful;

const MPSC_CHANNEL_READ_TIMEOUT: Duration = Duration::from_millis(50);

pub struct MinecraftServer {
    pub java_command: String,
    pub path_to_jar_dir: String,
    command: process::Command,
    handler: Option<McsrvHandler>,
}

const path_to_jar_grapher: (&'static str, &'static str) = ("/run/media/mark/Samsung_T5/Code/Java_IntelliJ/Java/Grapher/out/artifacts/Grapher_jar/", "Grapher.jar");
const path_to_jar_minecraft_local: (&'static str, &'static str) = ("/home/mark/Downloads/", "minecraft-server-1.19.2.jar");
const path_to_jar_minecraft_real: (&'static str, &'static str) = ("/media/mark/mcsrv/minecraft_server/1_19_0", "server.jar");
const path_to_jar_debug: (&'static str, &'static str) = ("/home/mark/Desktop/temporaeresprojektus/out/artifacts/temporaeresprojektus_jar/", "temporaeresprojektus.jar");
const path_to_jar: (&'static str, &'static str) = path_to_jar_minecraft_local;

impl MinecraftServer {
    pub fn new_default(config: DefaultServerConfig) -> Self {
        Self::new(
            "java".to_owned(),
            "-jar".to_owned(),
            match config {
                DefaultServerConfig::Grapher => path_to_jar_grapher.0.to_owned(),
                DefaultServerConfig::Mc1_19_0 => path_to_jar_minecraft_real.0.to_owned(),
                DefaultServerConfig::Mc1_19_2 => path_to_jar_minecraft_local.0.to_owned(),
            },
            match config {
                DefaultServerConfig::Grapher => path_to_jar_grapher.1.to_owned(),
                DefaultServerConfig::Mc1_19_0 => path_to_jar_minecraft_real.1.to_owned(),
                DefaultServerConfig::Mc1_19_2 => path_to_jar_minecraft_local.1.to_owned(),
            },
            match config {
                DefaultServerConfig::Grapher => vec![],
                DefaultServerConfig::Mc1_19_0 => vec!["--nogui"],
                DefaultServerConfig::Mc1_19_2 => vec!["--nogui"],
            }
        )
    }
    pub fn default_server_config_from_str(str: &str) -> Result<DefaultServerConfig, String> {
        match str {
            "Grapher" => Ok(DefaultServerConfig::Grapher),
            "1.19" => Ok(DefaultServerConfig::Mc1_19_0),
            "1.19.2" => Ok(DefaultServerConfig::Mc1_19_2),
            _ => Err(String::from("Only '1.19' and '1.19.2' are valid server types.")),
        }
    }
}
pub enum DefaultServerConfig {
    Grapher,
    Mc1_19_0,
    Mc1_19_2,
}

impl MinecraftServer {
    pub fn new<I, S>(java_command: String, java_jar_arg: String, path_to_jar_dir: String, name_of_jar: String, additional_args: I) -> Self where I: IntoIterator<Item = S>, S: AsRef<OsStr> {
        let mut command = process::Command::new(java_command.as_str());
        command.current_dir(path_to_jar_dir.as_str());
        command.arg(java_jar_arg.as_str()).arg(name_of_jar.as_str()).args(additional_args);
        command.stdout(Stdio::piped());
        command.stdin(Stdio::piped());
        command.stderr(Stdio::piped());
        Self {
            java_command,
            path_to_jar_dir,
            command,
            handler: None,
        }
    }

    /// Returns Ok(false) if the server is already running, Ok(true) if it was started, or an error if it could not be started.
    pub fn start(&mut self, ctx: Context, msg: Message) -> Result<bool, io::Error> {
        if self.handler.is_none() {
            self.handler = Some(McsrvHandler::new(self.command.spawn()?, ctx, msg));
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub fn kill(&mut self) -> bool {
        if let Some(handler) = &mut self.handler { handler.kill(); true } else { false }
    }

    pub fn write_to_stdin(&mut self, str: &str) -> io::Result<bool> {
        if let Some(handler) = &mut self.handler {
            let mut str = String::from(str);
            str.push('\n');
            handler.write_to_stdin(str.as_bytes())
        } else {
            Ok(false)
        }
    }
}

struct ProcessIOHandler {
    stderr: Option<ChildStderr>,
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
    stderr_sender: mpsc::Sender<u8>,
    stdin_sender: mpsc::Sender<u8>,
    stdout_sender: mpsc::Sender<u8>,
}
impl ProcessIOHandler {
    pub fn new(stderr: Option<ChildStderr>, stdin: Option<ChildStdin>, stdout: Option<ChildStdout>, stderr_sender: mpsc::Sender<u8>, stdin_sender: mpsc::Sender<u8>, stdout_sender: mpsc::Sender<u8>) -> Self {
        Self {
            stderr,
            stdin,
            stdout,
            stderr_sender,
            stdin_sender,
            stdout_sender,
        }
    }
    pub fn refresh_continuously(&mut self) {
        loop {

        }
    }
}