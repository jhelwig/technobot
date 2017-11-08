extern crate chrono;
extern crate discord;
extern crate dotenv;
#[macro_use]
extern crate nom;
extern crate rand;
extern crate reqwest;
extern crate regex;
extern crate scraper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod commands;
mod util;
mod framework;

use discord::Discord;
use discord::model::{Event, Message};
use dotenv::dotenv;
use framework::args::Args;
use std::env;

type Exec = Fn(&Discord, &Message, Args);

struct BotCommand {
    command: String,
    description: Option<String>,
    exec: Box<Exec>,
}

impl BotCommand {
    fn command_matches(&self, message: &str) -> bool {
        message.starts_with(&self.command)
    }
}

macro_rules! command (
    ($cmd_name:expr, $fn_name:path) => (
        {
            BotCommand {
                command: format!("{}", &$cmd_name).to_lowercase(),
                description: None,
                exec: Box::new($fn_name),
            }
        }
    );
    ($cmd_name:expr, $desc:expr, $fn_name:path) => (
        {
            BotCommand {
                command: format!("{}", &$cmd_name).to_lowercase(),
                description: Some($desc.to_string()),
                exec: Box::new($fn_name),
            }
        }
    );
);

fn main() {
    dotenv().ok();

    let bot_command_prefix = env::var("TECHNOBOT_PREFIX").unwrap_or("!".to_string());

    let mut commands: Vec<BotCommand> = Vec::new();
    commands.push(command!("ping", commands::misc::ping));
    commands.push(command!("8-ball",
                           "Ask the magic 8-ball any yes/no question.",
                           commands::misc::eight_ball));
    commands.push(command!("roll", commands::misc::roll));

    commands.push(command!("ffxiv resets",
                           "Show how long until the daily/weekly resets in FF XIV",
                           commands::ffxiv::resets));
    commands.push(command!("ffxiv events",
                           "List known events in FF XIV",
                           commands::ffxiv::events));

    let discord = match Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("Could not read DISCORD_TOKEN environment variable")) {
        Ok(d) => d,
        Err(e) => panic!("Unable to log in to Discord: {}", e),
    };

    let recommended_shards = match discord.suggested_shard_count() {
        Ok(s) => s,
        Err(e) => panic!("Could not get recommended shard count: {}", e),
    };
    println!("Recommended number of shards for bot: {}",
             &recommended_shards);

    let (mut connection, ready_event) = match discord.connect() {
        Ok((mut c, r)) => (c, r),
        Err(e) => panic!("Unable to connect to Discord: {}", e),
    };

    println!("Connected as: {}", &ready_event.user.username);
    match &ready_event.shard {
        &Some(s) => println!("Connected as shard: {:?}", s),
        &None => println!("Not using sharding."),
    }
    println!("Connected servers: {}", &ready_event.servers.len());

    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                if message.author.id != ready_event.user.id {
                    if message.content.starts_with(&bot_command_prefix) {
                        let lowercase_message = message.content[bot_command_prefix.len()..]
                            .to_lowercase();
                        for cmd in &commands {
                            if cmd.command_matches(&lowercase_message) {
                                (cmd.exec)(&discord,
                                           &message,
                                           Args::new(message.content[bot_command_prefix.len() +
                                                     cmd.command.len()..]
                                                             .trim()));
                                break;
                            }
                        }
                    }

                    if message.content == format!("{}help", &bot_command_prefix) {
                        for cmd in &commands {
                            let cmd_help = match &cmd.description {
                                &Some(ref d) => {
                                    format!("{}{} -> {}", &bot_command_prefix, &cmd.command, d)
                                }
                                &None => format!("{}{}", &bot_command_prefix, &cmd.command),
                            };
                            let _ = discord.send_message(message.channel_id, &cmd_help, "", false);
                        }
                    }
                    if message.content == format!("{}quit", &bot_command_prefix) {
                        println!("Quitting.");
                        break;
                    }
                }
            }
            Ok(_) => {}
            Err(discord::Error::Closed(code, body)) => {
                println!("Gateway closed on us with code {:?}: {}", code, body);
                break;
            }
            Err(err) => println!("Receive error: {:?}", err),
        }
    }
}
