use crate::{
    error::{Error, Location},
    lex::{Kind, Token},
    tree::{Schedule, Time},
};

pub fn parse(tokens: &[Token]) -> Result<Schedule, Error> {
    let Some(first) = tokens.first() else {
        return Err(Error::EmptySchedule);
    };

    if matches!(&first.kind, Kind::Word(word) if word == "Disable") {
        if let Some(extra) = tokens.get(1) {
            return Err(Error::UnexpectedToken { at: extra.at });
        }

        return Ok(Schedule { times: Vec::new() });
    }

    let mut times: Vec<Time> = Vec::new();

    for token in tokens {
        let Kind::Word(value) = &token.kind else {
            return Err(Error::UnexpectedToken { at: token.at });
        };

        let time = parse_time(value, token.at)?;

        if let Some(previous) = times.last()
            && time <= *previous
        {
            return Err(Error::UnsortedTimes {
                value: value.clone(),
                at: token.at,
            });
        }

        times.push(time);
    }

    Ok(Schedule { times })
}

fn parse_time(value: &str, at: Location) -> Result<Time, Error> {
    let invalid = || Error::InvalidTime {
        value: value.to_owned(),
        at,
    };

    let Some((hour, minute)) = value.split_once(':') else {
        return Err(invalid());
    };

    if !(1..=2).contains(&hour.len()) || minute.len() != 2 {
        return Err(invalid());
    }

    if !hour.bytes().all(|byte| byte.is_ascii_digit())
        || !minute.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(invalid());
    }

    let (Ok(hour), Ok(minute)) = (hour.parse::<u8>(), minute.parse::<u8>()) else {
        return Err(invalid());
    };

    if hour > 23 || minute > 59 {
        return Err(invalid());
    }

    Ok(Time { hour, minute })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lex;

    const HERE: Location = Location { line: 1, column: 1 };

    fn schedule(source: &str) -> Result<Schedule, Error> {
        parse(&lex(source).unwrap())
    }

    #[test]
    fn valid_time_formats() {
        assert_eq!(
            parse_time("8:00", HERE).unwrap(),
            Time { hour: 8, minute: 0 }
        );
        assert_eq!(
            parse_time("08:00", HERE).unwrap(),
            Time { hour: 8, minute: 0 }
        );
        assert_eq!(
            parse_time("23:59", HERE).unwrap(),
            Time {
                hour: 23,
                minute: 59
            }
        );
    }

    #[test]
    fn invalid_time_formats() {
        for value in ["8:0", "08:0", "800", "8:000", "25:00", "08:60", "+8:00", "ab:cd"] {
            assert!(parse_time(value, HERE).is_err(), "{value} should be rejected");
        }
    }

    #[test]
    fn valid_schedule() {
        let schedule = schedule(
            "
            // Morning
            08:00 09:00 09:55
            10:05 11:00 12:00

            // Afternoon
            14:00 15:00 15:55
            16:05 17:00 18:00
            ",
        )
        .unwrap();

        assert_eq!(schedule.times.len(), 12);
        assert_eq!(schedule.times[0], Time { hour: 8, minute: 0 });
        assert_eq!(
            schedule.times[11],
            Time {
                hour: 18,
                minute: 0
            }
        );
    }

    #[test]
    fn times_must_increase() {
        assert!(matches!(
            schedule("09:00 08:00"),
            Err(Error::UnsortedTimes {
                at: Location { line: 1, column: 7 },
                ..
            })
        ));

        assert!(matches!(
            schedule("08:00 08:00"),
            Err(Error::UnsortedTimes { .. })
        ));
    }

    #[test]
    fn disable() {
        assert_eq!(schedule("Disable").unwrap().times.len(), 0);
        assert_eq!(schedule("// off\nDisable\n").unwrap().times.len(), 0);
    }

    #[test]
    fn disable_cannot_mix_with_times() {
        assert!(schedule("Disable 08:00").is_err());
        assert!(schedule("08:00 Disable").is_err());
    }

    #[test]
    fn empty_schedule_is_an_error() {
        assert!(matches!(schedule(""), Err(Error::EmptySchedule)));
        assert!(matches!(
            schedule("// nothing here\n"),
            Err(Error::EmptySchedule)
        ));
    }

    #[test]
    fn braces_are_rejected() {
        assert!(schedule("{ 08:00 }").is_err());
    }
}
