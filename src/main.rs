#[macro_use]
extern crate serenity;
extern crate rand;

mod commands;

use serenity::prelude::*;
use serenity::model::*;
use serenity::framework::StandardFramework;

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
                          .configure(|c| c.prefix(&env::var("TECHNOBOT_PREFIX").unwrap_or("~".to_owned())))
                          .command("ping", |c| c.exec(commands::misc::ping))
                          .command("latency", |c| c.exec(commands::misc::latency))
                          .command("8-ball", |c| c.exec(commands::misc::eight_ball)));

    if let Err(why) = client.start_autosharded() {
        println!("Client error: {:?}", why);
    }
}
