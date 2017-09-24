#[macro_use]
extern crate serenity;

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
        .configure(|c| c.prefix("~"))
        .command("ping", |c| c.exec(commands::misc::ping))
        .command("latency", |c| c.exec(commands::misc::latency)));

    if let Err(why) = client.start_autosharded() {
        println!("Client error: {:?}", why);
    }
}
