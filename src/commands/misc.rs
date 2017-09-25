use rand;
use rand::Rng;

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
