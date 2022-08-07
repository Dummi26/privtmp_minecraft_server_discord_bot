use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serenity::{async_trait};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::discord_user_custom_permissions::DcUserPerms;
use crate::interaction_with_minecraft_server::MinecraftServer;

const prefix: &'static str = "mc.";
const RUN_AS_ADMINISTRATOR_HAHAHAHAHHAHAAHAHAH_I_HATE_MY_LIFE: &'static str = "https://media.discordapp.net/attachments/844851214760017970/1005224035598204948/runasadministrator.jpg";

pub struct DiscordInteractionsHandler {
    minecraft_servers: Arc<serenity::prelude::Mutex<HashMap<u64, Arc<Mutex<MinecraftServer>>>>>,
}
impl DiscordInteractionsHandler {
    pub fn new() -> Self {
        Self {
            minecraft_servers: Arc::new(serenity::prelude::Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EventHandler for DiscordInteractionsHandler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        let mut perms = DcUserPerms::new(msg.author.clone());
        let minecraft_servers_key = perms.get_user().id.0;
        let mut mcsrv = get_mcsrv(self, &minecraft_servers_key).await;
        async fn get_mcsrv(s: &DiscordInteractionsHandler, key: &u64) -> Option<Arc<Mutex<MinecraftServer>>> { let mut hm = s.minecraft_servers.lock().await; match hm.get_mut(key) { None => None, Some(v) => Some(v.clone()) } };
        if msg.content.starts_with(prefix) {
            let command_full = &msg.content[prefix.len()..];
            let mut commands_split_at_space = Vec::<String>::new(); {
                let mut command = String::new();
                for char in command_full.chars() {
                    if char == ' ' {
                        commands_split_at_space.push(command);
                        command = String::new();
                    } else {
                        command.push(char);
                    }
                };
                if command.len() != 0 { commands_split_at_space.push(command); }
            };
            if commands_split_at_space.len() != 0 {
                match commands_split_at_space[0].as_str() {
                    "start" => {
                        if mcsrv.is_none() || commands_split_at_space.len() >= 2 {
                            if commands_split_at_space.len() >= 2 {
                                let config = MinecraftServer::default_server_config_from_str(commands_split_at_space[1].as_str());
                                match config {
                                    Ok(config) => {
                                        if let Some(mcsrv) = mcsrv {
                                            match msg.reply(&ctx.http, "Killing the old server.").await { Ok(_) => {}, Err(err) => { println!("Could not send message! Error: {}", err); }, };
                                            mcsrv.lock().unwrap().kill();
                                        };
                                        if match config {
                                            crate::interaction_with_minecraft_server::DefaultServerConfig::Grapher => perms.is_admin(),
                                            crate::interaction_with_minecraft_server::DefaultServerConfig::Mc1_19_0 => perms.is_admin(),
                                            crate::interaction_with_minecraft_server::DefaultServerConfig::Mc1_19_2 => perms.is_admin(),
                                        } {
                                            let mut new_mcsrv = MinecraftServer::new_default(config);
                                            new_mcsrv.start(ctx, msg);
                                            match self.minecraft_servers.lock().await.insert(minecraft_servers_key, Arc::new(Mutex::new(new_mcsrv))) {
                                                Some(_) => println!("There was an old value"),
                                                None => println!("There was NO old value."),
                                            };
                                            mcsrv = get_mcsrv(self, &minecraft_servers_key).await;
                                        } else {
                                            match msg.reply(&ctx.http, "You do not have permission to start this kind of server.").await { Ok(_) => {}, Err(err) => { println!("Could not send message! Error: {}", err); }, };
                                        }
                                    },
                                    Err(error_msg) => {
                                        match msg.reply(&ctx.http, error_msg).await {
                                            Ok(_) => {},
                                            Err(err) => { println!("Could not send message! Error: {}", err); },
                                        };
                                    },
                                };
                            } else {
                                match msg.reply(&ctx.http, "You must specify what server you would like to start! Use 'help' to get a list of possibilities.").await {
                                    Ok(_) => {},
                                    Err(err) => { println!("Could not send message! Error: {}", err); },
                                };
                            }
                        };
                    },
                    "kill" => {
                        if let Some(mcsrv) = mcsrv {
                            if mcsrv.lock().unwrap().kill() {
                                match msg.reply(&ctx.http, "Forcibly stopping your server. This might take a few seconds.").await { Ok(_) => {}, Err(err) => { println!("Could not send message! Error: {}", err); }, };
                            } else {
                                match msg.reply(&ctx.http, "Server isn't running.").await { Ok(_) => {}, Err(err) => { println!("Could not send message! Error: {}", err); }, };
                            }
                        } else {
                            match msg.reply(&ctx.http, "No server bound to this channel.").await { Ok(_) => {}, Err(err) => { println!("Could not send message! Error: {}", err); }, };
                        };
                    }
                    "run" => {
                        if let Some(mcsrv) = mcsrv {
                            let what_to_run = &command_full["run".len()+1..];
                            mcsrv.lock().unwrap().write_to_stdin(what_to_run);
                        }
                    }
                    _ => {},
                }
            }
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