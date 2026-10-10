use crate::{
    error::Error,
    lex::Token,
    tree::{Schedule, Time},
};

pub fn parse(tokens: &[Token]) -> Result<Schedule, Error> {
    let mut times = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        let value = match &tokens[index] {
            Token::String(value) => value,
            _ => {
                return Err(Error::UnexpectedToken { position: index });
            }
        };

        times.push(parse_time(value, index)?);
        index += 1;

        if index == tokens.len() {
            break;
        }

        match tokens[index] {
            Token::Comma => {
                index += 1;

                if index == tokens.len() {
                    break;
                }
            }

            _ => {
                return Err(Error::UnexpectedToken { position: index });
            }
        }
    }

    Ok(Schedule { times })
}

fn parse_time(value: &str, position: usize) -> Result<Time, Error> {
    let Some((hour, minute)) = value.split_once(':') else {
        return Err(Error::InvalidTime {
            value: value.to_owned(),
            position,
        });
    };

    if !(1..=2).contains(&hour.len()) || minute.len() != 2 {
        return Err(Error::InvalidTime {
            value: value.to_owned(),
            position,
        });
    }

    let Ok(hour) = hour.parse::<u8>() else {
        return Err(Error::InvalidTime {
            value: value.to_owned(),
            position,
        });
    };

    let Ok(minute) = minute.parse::<u8>() else {
        return Err(Error::InvalidTime {
            value: value.to_owned(),
            position,
        });
    };

    if hour > 23 || minute > 59 {
        return Err(Error::InvalidTime {
            value: value.to_owned(),
            position,
        });
    }

    Ok(Time { hour, minute })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lex;

    #[test]
    fn valid_time_formats() {
        assert_eq!(parse_time("8:00", 0).unwrap(), Time { hour: 8, minute: 0 });

        assert_eq!(parse_time("08:00", 0).unwrap(), Time { hour: 8, minute: 0 });
    }

    #[test]
    fn invalid_time_formats() {
        assert!(parse_time("8:0", 0).is_err());
        assert!(parse_time("08:0", 0).is_err());
        assert!(parse_time("800", 0).is_err());
        assert!(parse_time("8:000", 0).is_err());
        assert!(parse_time("25:00", 0).is_err());
        assert!(parse_time("08:60", 0).is_err());
    }

    #[test]
    fn valid_schedule() {
        let tokens = lex(r#"
            "08:00", "09:00", "09:55",
            "10:05", "11:00", "12:00",

            "14:00", "15:00", "15:55",
            "16:05", "17:00", "18:00"
            "#)
        .unwrap();

        let schedule = parse(&tokens).unwrap();

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
    fn trailing_comma() {
        let tokens = lex(r#""08:00", "09:00","#).unwrap();

        let schedule = parse(&tokens).unwrap();

        assert_eq!(schedule.times.len(), 2);
    }

    #[test]
    fn missing_comma() {
        let tokens = lex(r#""08:00" "09:00""#).unwrap();

        assert!(parse(&tokens).is_err());
    }

    #[test]
    fn double_comma() {
        let tokens = lex(r#""08:00",,"#).unwrap();

        assert!(parse(&tokens).is_err());
    }
}
