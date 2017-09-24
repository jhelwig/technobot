use chrono::prelude::{DateTime, Utc, Weekday};
use chrono::Datelike;
use chrono::Duration;
use chrono::Timelike;
use serenity::framework::standard::CommandError;
use serenity::utils::Colour;

command!(resets(_ctx, msg) {
    let now = Utc::now();
    let daily_reset = next_daily_reset(now);
    let weekly_reset = next_weekly_reset(now);
    let crafting_reset = next_crafting_reset(now);

    let until_daily = until_string(daily_reset.signed_duration_since(now));
    let until_weekly = until_string(weekly_reset.signed_duration_since(now));
    let until_crafting = until_string(crafting_reset.signed_duration_since(now));

    let _ = msg.channel_id.send_message(|m| {
        m.embed(|e| {
            let mut embed = e.colour(Colour::rosewater()).title("FF XIV Resets");
            embed = embed.field(|f| { f.name("Daily").value(until_daily).inline(false) });
            embed = embed.field(|f| { f.name("Weekly").value(until_weekly).inline(false) });
            embed = embed.field(|f| { f.name("Crafting").value(until_crafting).inline(false) });

            embed
        })
    });
});

fn next_daily_reset(now: DateTime<Utc>) -> DateTime<Utc> {
    // Daily reset is every day at 15:00 UTC.
    let mut daily_reset: DateTime<Utc> = now.with_hour(15)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();
    if now.hour() >= 15 {
        daily_reset = daily_reset + Duration::days(1);
    }

    daily_reset
}

#[test]
fn next_daily_reset_when_before_reset_time() {
    let now: DateTime<Utc> = DateTime::parse_from_rfc3339("2017-09-24T04:00:00-00:00")
        .unwrap()
        .with_timezone(&Utc);
    let expected_reset: DateTime<Utc> = DateTime::parse_from_rfc3339("2017-09-24T15:00:00-00:00")
        .unwrap()
        .with_timezone(&Utc);
    let daily_reset = next_daily_reset(now);

    assert_eq!(daily_reset,
               expected_reset,
               "Expected next daily reset from {:?} to be at {:?}. Got: {:?}",
               now,
               expected_reset,
               daily_reset);
}

#[test]
fn next_daily_reset_when_after_reset_time() {
    let now: DateTime<Utc> = DateTime::parse_from_rfc3339("2017-09-30T15:00:00-00:00")
        .unwrap()
        .with_timezone(&Utc);
    let expected_reset: DateTime<Utc> = DateTime::parse_from_rfc3339("2017-10-01T15:00:00-00:00")
        .unwrap()
        .with_timezone(&Utc);
    let daily_reset = next_daily_reset(now);

    assert_eq!(daily_reset,
               expected_reset,
               "Expected next daily reset from {:?} to be at {:?}. Got: {:?}",
               now,
               expected_reset,
               daily_reset);
}

fn next_weekly_reset(now: DateTime<Utc>) -> DateTime<Utc> {
    // Weekly reset is every Tuesday at 08:00 UTC.
    let mut weekly_reset: DateTime<Utc> = now.with_hour(8)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();
    if now.weekday() == Weekday::Tue && now.hour() < 8 {
        return weekly_reset;
    }
    let current_weekday_number = now.weekday().number_from_monday();
    match current_weekday_number {
        2...7 => {
            weekly_reset = weekly_reset + Duration::days(i64::from(9 - current_weekday_number))
        }
        1 => weekly_reset = weekly_reset + Duration::days(1),
        _ => unreachable!(),
    }

    weekly_reset
}

#[test]
fn next_weekly_reset_before_reset_time_on_tuesday() {
    let now = DateTime::parse_from_rfc3339("2017-09-19T06:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let expected_reset = DateTime::parse_from_rfc3339("2017-09-19T08:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let weekly_reset = next_weekly_reset(now);

    assert_eq!(weekly_reset,
               expected_reset,
               "Expected next weekly reset from {:?} to be at {:?}. Got: {:?}",
               now,
               expected_reset,
               weekly_reset);
}

#[test]
fn next_weekly_reset_after_reset_time_on_tuesday() {
    let now = DateTime::parse_from_rfc3339("2017-09-19T08:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let expected_reset = DateTime::parse_from_rfc3339("2017-09-26T08:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let weekly_reset = next_weekly_reset(now);

    assert_eq!(weekly_reset,
               expected_reset,
               "Expected next weekly reset from {:?} to be at {:?}. Got: {:?}",
               now,
               expected_reset,
               weekly_reset);
}

#[test]
fn next_weekly_reset_on_monday() {
    let now = DateTime::parse_from_rfc3339("2017-09-18T06:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let expected_reset = DateTime::parse_from_rfc3339("2017-09-19T08:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let weekly_reset = next_weekly_reset(now);

    assert_eq!(weekly_reset,
               expected_reset,
               "Expected next weekly reset from {:?} to be at {:?}. Got: {:?}",
               now,
               expected_reset,
               weekly_reset);
}

#[test]
fn next_weekly_reset_on_wednesday() {
    let now = DateTime::parse_from_rfc3339("2017-09-20T06:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let expected_reset = DateTime::parse_from_rfc3339("2017-09-26T08:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let weekly_reset = next_weekly_reset(now);

    assert_eq!(weekly_reset,
               expected_reset,
               "Expected next weekly reset from {:?} to be at {:?}. Got: {:?}",
               now,
               expected_reset,
               weekly_reset);
}

fn next_crafting_reset(now: DateTime<Utc>) -> DateTime<Utc> {
    // Crafting reset is every Thursday at 08:00 UTC.
    let mut crafting_reset: DateTime<Utc> = now.with_hour(8)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();
    if now.weekday() == Weekday::Thu && now.hour() < 8 {
        return crafting_reset;
    }
    let current_weekday_number = now.weekday().number_from_monday();
    match current_weekday_number {
        4...7 => {
            crafting_reset = crafting_reset + Duration::days(i64::from(11 - current_weekday_number))
        }
        1...3 => {
            crafting_reset = crafting_reset + Duration::days(i64::from(4 - current_weekday_number))
        }
        _ => unreachable!(),
    }

    crafting_reset
}

#[test]
fn next_crafting_reset_before_reset_time_on_thursday() {
    let now = DateTime::parse_from_rfc3339("2017-09-21T06:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let expected_reset = DateTime::parse_from_rfc3339("2017-09-21T08:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let crafting_reset = next_crafting_reset(now);

    assert_eq!(crafting_reset,
               expected_reset,
               "Expected next crafting reset from {:?} to be at {:?}. Got: {:?}",
               now,
               expected_reset,
               crafting_reset);
}

#[test]
fn next_crafting_reset_after_reset_time_on_thursday() {
    let now = DateTime::parse_from_rfc3339("2017-09-28T08:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let expected_reset = DateTime::parse_from_rfc3339("2017-10-05T08:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let crafting_reset = next_crafting_reset(now);

    assert_eq!(crafting_reset,
               expected_reset,
               "Expected next crafting reset from {:?} to be at {:?}. Got: {:?}",
               now,
               expected_reset,
               crafting_reset);
}

#[test]
fn next_crafting_reset_on_wednesday() {
    let now = DateTime::parse_from_rfc3339("2017-09-20T06:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let expected_reset = DateTime::parse_from_rfc3339("2017-09-21T08:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let crafting_reset = next_crafting_reset(now);

    assert_eq!(crafting_reset,
               expected_reset,
               "Expected next crafting reset from {:?} to be at {:?}. Got: {:?}",
               now,
               expected_reset,
               crafting_reset);
}

#[test]
fn next_crafting_reset_on_friday() {
    let now = DateTime::parse_from_rfc3339("2017-09-22T06:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let expected_reset = DateTime::parse_from_rfc3339("2017-09-28T08:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let crafting_reset = next_crafting_reset(now);

    assert_eq!(crafting_reset,
               expected_reset,
               "Expected next crafting reset from {:?} to be at {:?}. Got: {:?}",
               now,
               expected_reset,
               crafting_reset);
}

fn until_string(until_duration: Duration) -> String {
    let mut components = Vec::new();
    let mut how_long = until_duration.clone();

    if how_long.num_weeks() > 0 {
        let week_str = if how_long.num_weeks() == 1 {
            "week"
        } else {
            "weeks"
        };
        components.push(format!("{} {}", how_long.num_weeks(), week_str));
    }
    how_long = how_long - Duration::weeks(how_long.num_weeks());

    if how_long.num_days() > 0 {
        let day_str = if how_long.num_days() == 1 {
            "day"
        } else {
            "days"
        };
        components.push(format!("{} {}", how_long.num_days(), day_str));
    }
    how_long = how_long - Duration::days(how_long.num_days());

    if how_long.num_hours() > 0 {
        let hour_str = if how_long.num_hours() == 1 {
            "hour"
        } else {
            "hours"
        };
        components.push(format!("{} {}", how_long.num_hours(), hour_str));
    }
    how_long = how_long - Duration::hours(how_long.num_hours());

    if how_long.num_minutes() > 0 {
        let minute_str = if how_long.num_minutes() == 1 {
            "minute"
        } else {
            "minutes"
        };
        components.push(format!("{} {}", how_long.num_minutes(), minute_str));
    }

    if components.is_empty() {
        components.push("less than a minute".to_string());
    }

    components.join(" ")
}

#[test]
fn until_string_with_one_week() {
    let now = DateTime::parse_from_rfc3339("2017-09-24T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event = DateTime::parse_from_rfc3339("2017-10-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let duration = event.signed_duration_since(now);
    let expected = "1 week";

    assert_eq!(until_string(duration), expected);
}

#[test]
fn until_string_with_multiple_weeks() {
    let now = DateTime::parse_from_rfc3339("2017-09-24T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event = DateTime::parse_from_rfc3339("2017-10-08T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let duration = event.signed_duration_since(now);
    let expected = "2 weeks";

    assert_eq!(until_string(duration), expected);
}

#[test]
fn until_string_with_one_day() {
    let now = DateTime::parse_from_rfc3339("2017-09-24T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event = DateTime::parse_from_rfc3339("2017-09-25T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let duration = event.signed_duration_since(now);
    let expected = "1 day";

    assert_eq!(until_string(duration), expected);
}

#[test]
fn until_string_with_multiple_days() {
    let now = DateTime::parse_from_rfc3339("2017-09-24T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event = DateTime::parse_from_rfc3339("2017-09-26T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let duration = event.signed_duration_since(now);
    let expected = "2 days";

    assert_eq!(until_string(duration), expected);
}

#[test]
fn until_string_with_one_hour() {
    let now = DateTime::parse_from_rfc3339("2017-09-24T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event = DateTime::parse_from_rfc3339("2017-09-24T01:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let duration = event.signed_duration_since(now);
    let expected = "1 hour";

    assert_eq!(until_string(duration), expected);
}

#[test]
fn until_string_with_multiple_hours() {
    let now = DateTime::parse_from_rfc3339("2017-09-24T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event = DateTime::parse_from_rfc3339("2017-09-24T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let duration = event.signed_duration_since(now);
    let expected = "2 hours";

    assert_eq!(until_string(duration), expected);
}

#[test]
fn until_string_with_one_minute() {
    let now = DateTime::parse_from_rfc3339("2017-09-24T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event = DateTime::parse_from_rfc3339("2017-09-24T00:01:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let duration = event.signed_duration_since(now);
    let expected = "1 minute";

    assert_eq!(until_string(duration), expected);
}

#[test]
fn until_string_with_multiple_minutes() {
    let now = DateTime::parse_from_rfc3339("2017-09-24T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event = DateTime::parse_from_rfc3339("2017-09-24T00:02:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let duration = event.signed_duration_since(now);
    let expected = "2 minutes";

    assert_eq!(until_string(duration), expected);
}

#[test]
fn until_string_with_multiple_components() {
    let now = DateTime::parse_from_rfc3339("2017-09-24T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event = DateTime::parse_from_rfc3339("2017-10-09T15:02:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let duration = event.signed_duration_since(now);
    let expected = "2 weeks 1 day 15 hours 2 minutes";

    assert_eq!(until_string(duration), expected);
}

#[test]
fn until_string_with_multiple_components_with_gap() {
    let now = DateTime::parse_from_rfc3339("2017-09-24T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event = DateTime::parse_from_rfc3339("2017-10-09T00:02:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let duration = event.signed_duration_since(now);
    let expected = "2 weeks 1 day 2 minutes";

    assert_eq!(until_string(duration), expected);
}

#[test]
fn until_string_with_less_than_a_minute() {
    let now = DateTime::parse_from_rfc3339("2017-09-24T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event = DateTime::parse_from_rfc3339("2017-09-24T00:00:30Z")
        .unwrap()
        .with_timezone(&Utc);
    let duration = event.signed_duration_since(now);
    let expected = "less than a minute";

    assert_eq!(until_string(duration), expected);
}
