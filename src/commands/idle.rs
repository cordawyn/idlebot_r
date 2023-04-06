use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub fn run(_options: &[CommandDataOption]) -> String {
    // TODO: Read the DB and retrieve a list of users,
    // ordered by idle time (most idle at the top).
    // Output the results as a string (message content).
    "List of idle users:".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("idle").description("List idle users")
}
