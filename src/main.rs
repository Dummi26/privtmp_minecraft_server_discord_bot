use serenity::prelude::*;

use handler::Handler;

mod secret;
mod handler;
mod useful;
mod interaction_with_minecraft_server;

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = secret::DISCORD_BOT_TOKEN.to_owned();
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
    ;

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler::new()).await.expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
