extern crate chrono;
extern crate futures;
extern crate hyper;
extern crate rand;
extern crate redis;
extern crate scraper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate serenity;
extern crate tokio_core;

mod commands;

//use redis;
use serenity::prelude::*;
use serenity::model::*;
use serenity::framework::StandardFramework;
use serenity::framework::standard::help_commands;

use std::cell::RefCell;
use std::env;

thread_local! {
    static REDIS_CLIENT: RefCell<redis::Client> = RefCell::new(
        redis::Client::open(
            env::var("TECHNOBOT_REDIS_CONNECTION")
                .unwrap_or("redis://127.0.0.1/".to_string())
                .as_str()
        ).unwrap()
    );
}

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
                                       .unwrap_or("~".to_string()))
                          })
                          .group("Final Fantasy XIV", |g| g
                                 .prefix("ffxiv")
                                 .command("resets", |c| {
                                     c.desc("Show how long until the daily/weekly/crafting resets in FF XIV")
                                         .exec(commands::ffxiv::resets)
                                 })
                                 .command("events", |c| {
                                     c.desc("List known events in FF XIV")
                                         .exec(commands::ffxiv::events)
                                 })
                          )
                          .command("ping", |c| c.exec(commands::misc::ping))
                          .command("latency", |c| c.exec(commands::misc::latency))
                          .command("8-ball", |c| c
                                   .desc("Ask the magic 8-ball any yes/no question")
                                   .exec(commands::misc::eight_ball)
                          )
                          .command("whatis", |c| c.exec(commands::dictionary::get_dictionary_entry))
                          .command("learn", |c| c.exec(commands::dictionary::set_dictionary_entry))
                          .command("forget", |c| c
                                   .exec(commands::dictionary::delete_dictionary_entry)
                                   .allowed_roles(vec!["technobot-dictionary-admin"])
                          )
                          .command("help", |c| c.exec_help(help_commands::with_embeds))
    );

    if let Err(why) = client.start_autosharded() {
        println!("Client error: {:?}", why);
    }
}
