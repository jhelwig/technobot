use discord::Discord;
use discord::model::Message;
use rand;
use rand::Rng;
use regex::Regex;
use framework::args::Args;

pub fn ping(bot: &Discord, msg: &Message, _args: Args) {
    let _ = bot.send_message(msg.channel_id, "Pong!", "", false);
}

pub fn eight_ball(bot: &Discord, msg: &Message, _args: Args) {
    let responses = ["It is certain",
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
                     "Very doubtful"];

    let _ = bot.send_message(msg.channel_id,
                             rand::thread_rng().choose(&responses).unwrap(),
                             "",
                             false);
}

pub fn roll(bot: &Discord, msg: &Message, mut args: Args) {
    let roll = match args.single::<String>() {
        Ok(r) => r,
        Err(_) => "1d1000".to_string(),
    };

    let re = Regex::new("^(?P<quantity>\\d+)?d(?P<sides>\\d+)$").expect("Couldn't create regex");
    let caps = match re.captures(&roll) {
        Some(c) => c,
        None => return,
    };

    let quantity = match caps.name("quantity") {
        Some(m) => {
            m.as_str()
                .parse::<u64>()
                .expect("Couldn't parse quantity of dice")
        }
        None => 1,
    };

    let sides = caps.name("sides")
        .unwrap()
        .as_str()
        .parse::<u64>()
        .expect("Couldn't parse how many sides per die");

    let mut total = 0;

    for _ in 0..quantity {
        total += rand::thread_rng().gen_range(0, sides) + 1;
    }

    let _ = bot.send_message(msg.channel_id,
                             &format!("{}: Rolled {} and got {}",
                                     msg.author.mention(),
                                     roll,
                                     total),
                             "",
                             false);
}
