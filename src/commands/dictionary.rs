use redis;
use redis::Commands;
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
    let key = args.single::<String>().unwrap();

    let result = execute_redis_command(RedisCommand::Get, &key, None::<&String>);

    let response = match result {
        Ok(r) => {
            match r {
                Some(value) => format!("{key} is {value}", key=key, value=value),
                None => format!("{key} is not set", key=key),
            }
        },
        Err(e) => {
            format!("Unable to retrieve {key}: {error}", key=key, error=e)
        },
    };

    let _ = msg.channel_id.say(response);
});

command!(set_dictionary_entry(_ctx, msg, args) {
    let key = args.single::<String>().unwrap();
    let value = args.single::<String>().unwrap();

    let result = execute_redis_command(RedisCommand::Set, &key, Some(&value));

    let response = match result {
        Ok(_) => {
            format!("Ok! {key} is now {value}", key=key, value=value)
        },
        Err(e) => {
            format!("Unable to set {key} to {value}: {error}", key=key, value=value, error=e)
        },
    };

    let _ = msg.channel_id.say(response);
});

fn execute_redis_command<'a, T>(cmd: RedisCommand, key: &String, val: Option<&'a T>) -> Result<Option<String>, String> where &'a T: redis::ToRedisArgs{
    REDIS_CONNECTION.with(|r| {
        let con: &redis::RedisResult<redis::Connection> = &*r.borrow();
        let full_key = namespaced_dictionary_key(&key);

        match con {
            &Ok(ref c) => {
                match cmd {
                    RedisCommand::Get => {
                        match c.exists(full_key.clone()) {
                            Ok(true) => {
                                match c.get(full_key) {
                                    Ok(r) => Ok(Some(r)),
                                    Err(e) => Err(e.description().to_string()),
                                }
                            },
                            Ok(false) => Ok(None),
                            Err(e) => Err(e.description().to_string()),
                        }
                    },
                    RedisCommand::Set => {
                        match c.set(full_key, val) {
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

fn namespaced_dictionary_key(key: &String) -> String {
    format!("technobot::dictionary::{}", key)
}
