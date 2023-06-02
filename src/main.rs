mod commands;

use std::env;
use std::fs::File;
use std::io::Read;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::id::GuildId;
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
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.author.bot {
            // GuildId is None when we're dealing with a private message
            if let Some(gid) = msg.guild_id {
                println!("Guild id: {}", gid);
                let mut data = ctx.data.write().await;
                let db = data.get_mut::<DatabaseConnection>().unwrap();
                if let Ok(timestamps) = db.open_tree(gid.to_string()) {
                    let aid = msg.author.id.as_u64();
                    let ts = msg.timestamp.unix_timestamp();
                    timestamps.insert(aid.to_be_bytes(), &ts.to_be_bytes()).expect("ERROR: Could not insert timestamp data!");
                }
                // Store AuthorId -> Author Nickname reference
                // to avoid looking it up when responding with a list of "idle users".
                if let Ok(authors) = db.open_tree("authors") {
                    let aid = msg.author.id.as_u64();
                    let aname = msg.author.name;
                    authors.insert(aid.to_be_bytes(), aname.as_bytes()).expect("ERROR: Could not insert user data!");
                }
            }
        }
    }

    // TODO: Handle "slash-commands" to allow interactions with the bot:
    // "/idle" - list of idle users (nickname + idle time), longest idle at the top.
    // "/prune [N]" - remove records of idle users who have been idle N time (days?); N can be optional, with some default value;
    //                creates a "recover" copy of the current DB.
    // "/recover" - recovers the DB (copies a recover DB, if it is available, over the current DB).
    // All commands MUST be scoped to the current guild (hopefully, its id is sent with the slash command).
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let gid = command.data.guild_id.expect("No guild id somehow.");

            let mut data = ctx.data.write().await;
            let db = data.get_mut::<DatabaseConnection>().unwrap();

            let content = match command.data.name.as_str() {
                "idle" => commands::idle::run(&command.data.options, db, &gid),
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, ctx: Context, ready: Ready) {
        // Register slash commands

        // FIXME: Temporarily, register as a guild command, then make it "global" (see below)
        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let _commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::idle::register(command))
        })
        .await;

        // TODO: "Convert" guild commands above into global commands when done testing
        // let guild_command = Command::create_global_application_command(&ctx.http, |command| {
        //     commands::wonderful_command::register(command)
        // })
        // .await;

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

struct DatabaseConnection;

impl TypeMapKey for DatabaseConnection {
    type Value = sled::Db;
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

    // Setup a persistent DB connection for the lifetime of the server.
    // TODO: flush/clean-up on server shutdown?
    {
        let mut data = client.data.write().await;
        data.insert::<DatabaseConnection>(sled::open("database").expect("ERROR: Could not open the database!"));
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
