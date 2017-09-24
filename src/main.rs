extern crate chrono;
extern crate rand;
#[macro_use]
extern crate serenity;

mod commands;

use serenity::prelude::*;
use serenity::model::*;
use serenity::framework::StandardFramework;
use serenity::framework::standard::help_commands;

use std::env;

struct Handler;
impl EventHandler for Handler {
    fn on_ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    let mut client = Client::new(&env::var("DISCORD_TOKEN").unwrap(), Handler);

    client.with_framework(StandardFramework::new()
                          .configure(|c| {
                              c.prefix(&env::var("TECHNOBOT_PREFIX")
                                       .unwrap_or("~".to_owned()))
                          })
                          .group("Final Fantasy XIV", |g| g
                                 .prefix("ffxiv")
                                 .command("resets", |c| {
                                     c.desc("Show how long until the daily/weekly/crafting resets in FF XIV")
                                         .help_available(true)
                                         .exec(commands::ffxiv::resets)
                                 })
                          )
                          .command("ping", |c| c.exec(commands::misc::ping))
                          .command("latency", |c| c.exec(commands::misc::latency))
                          .command("8-ball", |c| c
                                   .desc("Ask the magic 8-ball any yes/no question")
                                   .exec(commands::misc::eight_ball)
                          )
                          .command("help", |c| c.exec_help(help_commands::with_embeds))
    );

    if let Err(why) = client.start_autosharded() {
        println!("Client error: {:?}", why);
    }
}
