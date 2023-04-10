use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::GuildId;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use sled::Db;

pub fn run(_options: &[CommandDataOption], db: &mut Db, gid: &GuildId) -> String {
    if let Ok(tree) = db.open_tree(gid.to_string()) {
        // TODO: Read the DB and retrieve a list of users,
        // ordered by idle time (most idle at the top).
        for rec in tree.iter() {
            let (k, v) = rec.unwrap();
            println!("Record: {:#?} {:#?}",
                k.to_vec(),
                v.to_vec());
        }
    }
    // Output the results as a string (message content).
    "List of idle users:".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("idle").description("List idle users")
}
