use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serenity::{async_trait};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::interaction_with_minecraft_server::MinecraftServer;

const prefix: &'static str = "[mc].";
const discord_id_owner: u64 = 402830098870435857;
const RUN_AS_ADMINISTRATOR_HAHAHAHAHHAHAAHAHAH_I_HATE_MY_LIFE: &'static str = "https://media.discordapp.net/attachments/844851214760017970/1005224035598204948/runasadministrator.jpg";

pub struct Handler {
    minecraft_server: Arc<Mutex<MinecraftServer>>,
}
impl Handler {
    pub fn new() -> Self {
        Self {
            minecraft_server: Arc::new(Mutex::new(MinecraftServer::new(
                MinecraftServer::default_java_command(), MinecraftServer::default_java_jar_arg(),
                MinecraftServer::default_dir_of_jar(), MinecraftServer::default_name_of_jar(),
                MinecraftServer::default_additional_args()
            ))),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with(prefix) {
            println!("{} ({}) @ {}:\n{}", msg.author.name, msg.author.id, msg.timestamp, msg.content);
            let command = &msg.content[prefix.len()..];
            let index_of_end_of_command_identifier = command.find(' ');
            match &command[0 .. match index_of_end_of_command_identifier { None => command.len(), Some(index) => index, }] {
                "run" => {
                    if let Some(index_of_end_of_command_identifier) = index_of_end_of_command_identifier {
                        if index_of_end_of_command_identifier+1 < command.len() {
                            if msg.author.id.0 == discord_id_owner {
                                let other_stuff = &command[index_of_end_of_command_identifier+1..];
                                let bytes = other_stuff.as_bytes();
                                let mut errored = false;
                                for byte in bytes {
                                    match self.minecraft_server.lock().await.get_cloned_stdin_sender().send(*byte) {
                                        Ok(_) => {},
                                        Err(_) => { errored = true; },
                                    }
                                };
                                match msg.reply(&ctx.http, if errored { "There was an error while sending the command. It is possible that none (or only some) of the command has reached the child process." } else { "Done. Your command should be executed as soon as the child process reads it." }).await {
                                    Ok(_) => {},
                                    Err(_) => {},
                                }
                            } else {
                                match msg.reply(&ctx.http, "You must either run this command as administrator (owner) or without any arguments.").await {
                                    Ok(_) => {},
                                    Err(_) => {}, // this is litterally a meme, i'm not putting an error message here
                                }
                            }
                        } else {
                            match msg.reply(&ctx.http, RUN_AS_ADMINISTRATOR_HAHAHAHAHHAHAAHAHAH_I_HATE_MY_LIFE).await {
                                Ok(_) => {},
                                Err(_) => {}, // this is litterally a meme, i'm not putting an error message here
                            };
                        };
                    } else {
                        match msg.reply(&ctx.http, RUN_AS_ADMINISTRATOR_HAHAHAHAHHAHAAHAHAH_I_HATE_MY_LIFE).await {
                            Ok(_) => {},
                            Err(_) => {}, // this is litterally a meme, i'm not putting an error message here
                        };
                    };
                }
                "start" => {
                    if msg.author.id.0 == discord_id_owner {
                        println!("Starting...");
                        match self.minecraft_server.lock().await.start() {
                            Ok(_) => {
                                match msg.reply(&ctx.http, "Server is starting...").await {
                                    Ok(mut msg) => {
                                        let ctx = ctx.clone();
                                        let mcsrc_mutex = self.minecraft_server.clone();
                                        thread::spawn(move || {
                                            let sleep_duration = Duration::from_secs(3);
                                            let mpsc_timeout = Duration::from_millis(100);
                                            loop {
                                                thread::sleep(sleep_duration);
                                                let mut mcsrv = mcsrc_mutex.blocking_lock();
                                                mcsrv.update_process_stdio();
                                                match futures::executor::block_on(msg.edit(&ctx.http, |m| m.content(format!("STDOUT:\n```\n{}\n```", mcsrv.get_stdout_last_500_chars())))) {
                                                    Ok(_) => {},
                                                    Err(err) => { println!("Error editing message: {}", err); }
                                                };
                                            };
                                        });
                                    },
                                    Err(err) => { println!("Error sending message reply to user {}: {}", msg.author, err); },
                                };
                            },
                            Err(err) => {
                                println!("Failed to start child process: {}", err);
                                match msg.reply(&ctx.http, "Server could not be started. Check stdout to see the internal error.").await {
                                    Ok(_) => {},
                                    Err(_) => {},
                                };
                            },
                        };
                    } else {
                        match msg.reply(&ctx.http, "You do not have permission to execute this command, consider hijacking the owner's discord account and trying again.").await {
                            Ok(_) => {},
                            Err(err) => { println!("Error sending message reply to user {}: {}", msg.author, err); },
                        }
                    };
                },
                "kill" => {
                    self.minecraft_server.lock().await.write_stdout_and_stderr_to_file(); // for debugging later
                    match self.minecraft_server.lock().await.kill() {
                        None => {
                            match msg.reply(&ctx.http, "There was no process to kill.").await {
                                Ok(_) => {},
                                Err(err) => { println!("Error sending message reply to user {}: {}", msg.author, err); },
                            }
                        },
                        Some(v) => {
                            match v {
                                Ok(v) => {
                                    match msg.reply(&ctx.http, "Killed the child process.").await {
                                        Ok(_) => {},
                                        Err(err) => { println!("Error sending message reply to user {}: {}", msg.author, err); },
                                    }
                                },
                                Err(err) => {
                                    println!("Failed to kill child process: {}", err);
                                    match msg.reply(&ctx.http, "Child did not want to die. (failed to kill child process - check hardware log for details)").await {
                                        Ok(_) => {},
                                        Err(err) => { println!("Error sending message reply to user {}: {}", msg.author, err); },
                                    }
                                },
                            }
                        },
                    }
                }
                _ => {},
            };
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}