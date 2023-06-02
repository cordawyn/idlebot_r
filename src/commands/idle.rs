use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::GuildId;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::timestamp::Timestamp;
use sled::Db;
use std::collections::BTreeMap;
use std::time::Duration;
use human_repr::HumanDuration;


pub fn run(_options: &[CommandDataOption], db: &mut Db, gid: &GuildId) -> String {
    let mut sorted_authors: BTreeMap<i64, String> = BTreeMap::new();

    if let Ok(tree) = db.open_tree(gid.to_string()) {
        if let Ok(authors) = db.open_tree("authors") {
            for rec in tree.iter() {
                let (k, v) = rec.unwrap();
    
                let ts_bv = <[u8; 8]>::try_from(v.as_ref()).unwrap();
                let unix_ts = i64::from_be_bytes(ts_bv);
                let aid_bv = <[u8; 8]>::try_from(k.as_ref()).unwrap();
                // let aid = UserId::try_from(u64::from_be_bytes(aid_bv)).unwrap();
                let name = match authors.get(&aid_bv).unwrap() {
                    Some(n) => String::from_utf8(n.to_vec()).unwrap(),
                    None => String::from("Unknown")
                };
    
                sorted_authors.insert(unix_ts, name);
            }
        }
    }

    let mut aus = String::new();
    for (unix_ts, name) in sorted_authors.iter() {
        let ts = Timestamp::from_unix_timestamp(*unix_ts).unwrap();
        let delta = Timestamp::now().unix_timestamp() - ts.unix_timestamp();
        let entry = format!("* {} ({})\n", name, Duration::from_secs(delta.try_into().unwrap()).human_duration().to_string());
        aus.push_str(&entry);
    }

    // Output the results as a string (message content).
    format!("List of idle users:\n{aus}")
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("idle").description("List idle users")
}
