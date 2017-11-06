use chrono::prelude::{DateTime, Utc, Weekday};
use chrono::{Datelike, Duration, Timelike};
use reqwest;
use scraper::{Html, Selector};
use serde_json;
use serenity::utils::Colour;
use std::io::Read;

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

command!(events(_ctx, msg) {
    let now = Utc::now();

    let events = get_events();
    match events {
        Ok(even) => {
            for event in even.iter() {
                if event.end > now {
                    let _ = msg.channel_id.send_message(|m| {
                        m.embed(|e| {
                            let mut embed = e.colour(Colour::rosewater()).title(event.name());
                            embed = embed.field(|f| { f.name("More information").value(event.url()).inline(false) });
                            if event.start > now {
                                embed = embed.field(|f| { f.name("Start").value(until_string(event.start.signed_duration_since(now))).inline(false) });
                            };
                            embed = embed.field(|f| { f.name("End").value(until_string(event.end.signed_duration_since(now))).inline(false) });
                            if let Some(ref info) = event.info {
                                embed = embed.field(|f| { f.name("Info").value(info) });
                            }

                            embed
                        })
                    });
                }
            }
        },
        Err(e) => { println!("{}", e); },
    };
});

fn get_events() -> Result<Vec<FFXIVEvent>, String> {
    let json = retrieve_event_json();
    let timers;
    match json {
        Ok(j) => { timers = parse_event_json(&j) },
        Err(e) => return Err(format!("{}", e)),
    };

    match timers {
        Ok(t) => Ok(t.events),
        Err(e) => Err(format!("{}", e)),
    }
}

fn retrieve_event_json() -> Result<String, String> {
    let url = "http://www.xenoveritas.org/static/ffxiv/timers.json";
    let mut resp = match reqwest::get(url) {
        Ok(r) => r,
        Err(e) => return Err(format!("{}", e)),
    };

    if !resp.status().is_success() {
        return Err(format!("Unable to fetch {}: {}", url, resp.status()));
    }

    let mut content = String::new();
    if let Err(e) = resp.read_to_string(&mut content) {
        return Err(format!("Invalid content: {}", e));
    }
    Ok(content)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct FFXIVTimers {
    #[serde(rename = "timers")]
    pub events: Vec<FFXIVEvent>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct FFXIVEvent {
    #[serde(rename = "name")]
    pub name_html: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(with = "javascript_date_format")]
    pub start: DateTime<Utc>,
    #[serde(with = "javascript_date_format")]
    pub end: DateTime<Utc>,
    pub info: Option<String>,
}

impl FFXIVEvent {
    pub fn name(&self) -> String {
        let fragment = Html::parse_fragment(&self.name_html);
        let selector = Selector::parse("a").unwrap();

        let link = fragment.select(&selector).next().unwrap();

        link.inner_html()
    }

    pub fn url(&self) -> String {
        let fragment = Html::parse_fragment(&self.name_html);
        let selector = Selector::parse("a").unwrap();

        let link = fragment.select(&selector).next().unwrap();

        link.value().attr("href").unwrap().to_string()
    }
}

mod javascript_date_format {
    use chrono::prelude::{DateTime, Utc};
    use chrono::naive::NaiveDateTime;
    use serde::{Deserialize, Serializer, Deserializer};

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error> where S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let s = format!("{}", date.format("%s000"));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<D>(D) -> Result<T, D::Error> where D: Deserializer
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
        where D: Deserializer<'de>
    {
        let stamp = f64::deserialize(deserializer)? as i64;
        //Utc.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
        Ok(DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(stamp / 1000, 0), Utc))
    }
}

fn parse_event_json(event_json: &str) -> Result<FFXIVTimers, String> {
    let result = serde_json::from_str(event_json);
    match result {
        Ok(r) => Ok(r),
        Err(e) => Err(format!("{}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn parse_event_json_with_info() {
        let json_to_parse = r#"{"timers":[{"name":"<a href=\"http://na.finalfantasyxiv.com/mount_campaign_2017/\">Fly the Falcon Mount Campaign</a>","type":"campaign","start":14988924E5,"end":150684114E4,"info":"Players who purchase a total of 90 days of subscription time during this time period will receive a Falcon mount."}]}"#;
        let expected = FFXIVTimers {
            events: vec![
                FFXIVEvent {
                    name_html: "<a href=\"http://na.finalfantasyxiv.com/mount_campaign_2017/\">Fly the Falcon Mount Campaign</a>".to_string(),
                    kind: "campaign".to_string(),
                    start: DateTime::parse_from_rfc3339("2017-07-01T07:00:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    end: DateTime::parse_from_rfc3339("2017-10-01T06:59:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    info: Some("Players who purchase a total of 90 days of subscription time during this time period will receive a Falcon mount.".to_string())
                }
            ]
        };

        let result: FFXIVTimers = parse_event_json(&json_to_parse).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn parse_event_json_without_info() {
        let json_to_parse = r#"{"timers":[{"name":"<a href=\"http://na.finalfantasyxiv.com/lodestone/special/2017/The_Rising/\">The Rising</a>","type":"event rising","start":15037596E5,"end":150540114E4,"showDuration":true}]}"#;
        let expected = FFXIVTimers {
            events: vec![
                FFXIVEvent {
                    name_html: "<a href=\"http://na.finalfantasyxiv.com/lodestone/special/2017/The_Rising/\">The Rising</a>".to_string(),
                    kind: "event rising".to_string(),
                    start: DateTime::parse_from_rfc3339("2017-08-26T15:00:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    end: DateTime::parse_from_rfc3339("2017-09-14T14:59:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    info: None
                }
            ]
        };

        let result: FFXIVTimers = parse_event_json(&json_to_parse).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn parse_event_json_with_multiple_events() {
        let json_to_parse = r#"{"timers":[{"name":"<a href=\"http://na.finalfantasyxiv.com/mount_campaign_2017/\">Fly the Falcon Mount Campaign</a>","type":"campaign","start":14988924E5,"end":150684114E4,"info":"Players who purchase a total of 90 days of subscription time during this time period will receive a Falcon mount."},{"name":"<a href=\"http://na.finalfantasyxiv.com/lodestone/special/2017/The_Rising/\">The Rising</a>","type":"event rising","start":15037596E5,"end":150540114E4,"showDuration":true},{"name":"<a href=\"https://na.finalfantasyxiv.com/lodestone/special/2017/youkai-watch/\">Yo-kai Watch</a>","type":"event yokai-watch","start":15044256E5,"end":150954834E4,"showDuration":true},{"name":"<a href=\"http://na.finalfantasyxiv.com/lodestone/news/detail/5c652e0bf15a5028c3d2762c78d9ed094c475472\">All Worlds Maintenance (Sep. 14)</a>","type":"maintenance","start":15054552E5,"end":1505466E6}]}"#;
        let expected = FFXIVTimers {
            events: vec![
                FFXIVEvent {
                    name_html: "<a href=\"http://na.finalfantasyxiv.com/mount_campaign_2017/\">Fly the Falcon Mount Campaign</a>".to_string(),
                    kind: "campaign".to_string(),
                    start: DateTime::parse_from_rfc3339("2017-07-01T07:00:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    end: DateTime::parse_from_rfc3339("2017-10-01T06:59:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    info: Some("Players who purchase a total of 90 days of subscription time during this time period will receive a Falcon mount.".to_string())
                },
                FFXIVEvent {
                    name_html: "<a href=\"http://na.finalfantasyxiv.com/lodestone/special/2017/The_Rising/\">The Rising</a>".to_string(),
                    kind: "event rising".to_string(),
                    start: DateTime::parse_from_rfc3339("2017-08-26T15:00:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    end: DateTime::parse_from_rfc3339("2017-09-14T14:59:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    info: None
                },
                FFXIVEvent {
                    name_html: "<a href=\"https://na.finalfantasyxiv.com/lodestone/special/2017/youkai-watch/\">Yo-kai Watch</a>".to_string(),
                    kind: "event yokai-watch".to_string(),
                    start: DateTime::parse_from_rfc3339("2017-09-03T08:00:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    end: DateTime::parse_from_rfc3339("2017-11-01T14:59:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    info: None
                },
                FFXIVEvent {
                    name_html: "<a href=\"http://na.finalfantasyxiv.com/lodestone/news/detail/5c652e0bf15a5028c3d2762c78d9ed094c475472\">All Worlds Maintenance (Sep. 14)</a>".to_string(),
                    kind: "maintenance".to_string(),
                    start: DateTime::parse_from_rfc3339("2017-09-15T06:00:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    end: DateTime::parse_from_rfc3339("2017-09-15T09:00:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    info: None
                }
            ]
        };

        let result: FFXIVTimers = parse_event_json(&json_to_parse).unwrap();

        assert_eq!(result, expected);
    }
}
