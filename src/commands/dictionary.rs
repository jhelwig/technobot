use redis;
use redis::Commands;
use serenity::model::GuildId;
use std::cell::RefCell;
use std::error::Error;

enum RedisCommand {
    Get,
    Set,
}

thread_local! {
    static REDIS_CONNECTION: RefCell<redis::RedisResult<redis::Connection>> = RefCell::new(
        ::REDIS_CLIENT.with(
            |r| r.borrow_mut().get_connection()
        )
    );
}

command!(get_dictionary_entry(_ctx, msg, args) {
    let guild_id = msg.guild_id();

    let short_key = args.single::<String>().unwrap();
    let key;
    match guild_id {
        None => {
            let _ = msg.channel_id.say("Sorry, I can't do that here.");
            return Ok(());
        },
        Some(GuildId(g)) => {
            key = namespaced_dictionary_key(&short_key, g);
        },
    };

    let result = execute_redis_command(RedisCommand::Get, &key, None::<&String>);

    let response = match result {
        Ok(r) => {
            match r {
                Some(value) => format!("{key} is {value}", key=short_key, value=value),
                None => format!("{key} is not set", key=short_key),
            }
        },
        Err(e) => {
            format!("Unable to retrieve {key}: {error}", key=short_key, error=e)
        },
    };

    let _ = msg.channel_id.say(response);
});

command!(set_dictionary_entry(_ctx, msg, args) {
    let guild_id = msg.guild_id();

    let short_key = args.single::<String>().unwrap();
    let key;
    match guild_id {
        None => {
            let _ = msg.channel_id.say("Sorry, I can't do that here.");
            return Ok(());
        },
        Some(GuildId(g)) => {
            key = namespaced_dictionary_key(&short_key, g);
        },
    };
    let value = args.single::<String>().unwrap();

    let result = execute_redis_command(RedisCommand::Set, &key, Some(&value));

    let response = match result {
        Ok(_) => {
            format!("Ok! {key} is now {value}", key=short_key, value=value)
        },
        Err(e) => {
            format!("Unable to set {key} to {value}: {error}", key=short_key, value=value, error=e)
        },
    };

    let _ = msg.channel_id.say(response);
});

fn execute_redis_command<'a, T>(cmd: RedisCommand, key: &String, val: Option<&'a T>) -> Result<Option<String>, String> where &'a T: redis::ToRedisArgs{
    REDIS_CONNECTION.with(|r| {
        let con: &redis::RedisResult<redis::Connection> = &*r.borrow();

        match con {
            &Ok(ref c) => {
                match cmd {
                    RedisCommand::Get => {
                        match c.exists(key.clone()) {
                            Ok(true) => {
                                match c.get(key) {
                                    Ok(r) => Ok(Some(r)),
                                    Err(e) => Err(e.description().to_string()),
                                }
                            },
                            Ok(false) => Ok(None),
                            Err(e) => Err(e.description().to_string()),
                        }
                    },
                    RedisCommand::Set => {
                        match c.set(key, val) {
                            Ok(()) => Ok(None),
                            Err(e) => Err(e.description().to_string()),
                        }
                    },
                }
            },
            &Err(ref e) => Err(e.description().to_string()),
        }
    })
}

fn namespaced_dictionary_key(key: &String, guild_id: u64) -> String {
    format!("technobot::dictionary::{guild_id}::{key}", key=key, guild_id=guild_id)
}
