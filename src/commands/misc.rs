use rand;
use rand::Rng;
use regex::Regex;

command!(latency(ctx, msg) {
    let latency = ctx.shard.lock()
                           .latency()
                           .map_or_else(|| "N/A".to_owned(), |s| {
                               format!("{}.{}s", s.as_secs(), s.subsec_nanos())
                           });

    let _ = msg.channel_id.say(latency);
});

command!(ping(_ctx, msg) {
    let _ = msg.channel_id.say("Pong!");
});

command!(eight_ball(_ctx, msg) {
    let responses = [
        "It is certain",
        "It is decidedly so",
        "Without a doubt",
        "Yes definitely",
        "You may rely on it",
        "As I see it, yes",
        "Most likely",
        "Outlook good",
        "Yes",
        "Signs point to yes",
        "Reply hazy try again",
        "Ask again later",
        "Better not tell you now",
        "Cannot predict now",
        "Concentrate and ask again",
        "Don't count on it",
        "My reply is no",
        "My sources say no",
        "Outlook not so good",
        "Very doubtful"
    ];

    let _ = msg.channel_id.say(rand::thread_rng().choose(&responses).unwrap());
});

command!(dice(_ctx, msg, arg) {
    let roll = arg.single::<String>().unwrap();
    let re = Regex::new("^(?P<quantity>\\d+)?d(?P<sides>\\d+)$").unwrap();
    let caps = match re.captures(&roll) {
        Some(c) => c,
        None => return Ok(()),
    };

    let quantity = match caps.name("quantity") {
        Some(m) => m.as_str().parse::<u64>().unwrap(),
        None => 1,
    };

    let sides = caps.name("sides").unwrap().as_str().parse::<u64>().unwrap();

    let mut total = 0;

    for _ in 0..quantity {
        total += rand::thread_rng().gen_range(0, sides) + 1;
    }

    let _ = msg.channel_id.say(format!("Rolled {} and got {}", roll, total));
});
