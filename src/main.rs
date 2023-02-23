use std::fs::File;
use std::io::Read;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use toml::from_str;
use serde::Deserialize;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, _ctx: Context, msg: Message) {
        // TODO
        if !msg.author.bot {
            println!("Author Id: {:?}", msg.author.id);
            println!("Timestamp: {:?}", msg.timestamp);
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

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "DISCORD_TOKEN")]
    discord_token: String
}

fn configure() -> Config {
    // Configure the client with your Discord bot token in the environment.
    let mut file = File::open("Secrets.toml").unwrap();
    let mut config = String::new();
    file.read_to_string(&mut config).unwrap();
    return from_str(&config).unwrap();
}

#[tokio::main]
async fn main() {
    let token = configure().discord_token;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES;

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
