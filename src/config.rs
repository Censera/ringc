use crate::{
    error::Error,
    lex::Token,
    tree::{Config, ConfigEntry, Date, Rule},
};

pub fn parse(tokens: &[Token]) -> Result<Config, Error> {
    let mut entries = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        let name = match &tokens[index] {
            Token::Identifier(name) => name.clone(),
            _ => {
                return Err(Error::UnexpectedToken { position: index });
            }
        };

        index += 1;

        if !matches!(tokens.get(index), Some(Token::LeftBrace)) {
            return Err(Error::UnexpectedToken { position: index });
        }

        index += 1;

        let mut rules = Vec::new();

        loop {
            match tokens.get(index) {
                Some(Token::Identifier(rule)) => {
                    let rule = match rule.as_str() {
                        "default" => Rule::Default,
                        "automatic" => Rule::Automatic,
                        "always" => Rule::Always,
                        _ => {
                            return Err(Error::UnexpectedToken { position: index });
                        }
                    };

                    rules.push(rule);
                    index += 1;
                }

                Some(Token::String(from)) => {
                    let position = index;
                    let from = parse_date(from, position)?;
                    index += 1;
                
                    if !matches!(tokens.get(index), Some(Token::Range)) {
                        return Err(Error::UnexpectedToken { position: index });
                    }
                
                    index += 1;
                
                    let to_position = index;
                
                    let to = match tokens.get(index) {
                        Some(Token::String(to)) => parse_date(to, to_position)?,
                        _ => {
                            return Err(Error::UnexpectedToken { position: index });
                        }
                    };
                
                    if !is_concrete(&from) {
                        return Err(Error::UnexpectedToken { position });
                    }
                
                    if !is_concrete(&to) {
                        return Err(Error::UnexpectedToken {
                            position: to_position,
                        });
                    }

                    if (
                        from.year.unwrap(),
                        from.month.unwrap(),
                        from.day.unwrap(),
                    ) > (
                        to.year.unwrap(),
                        to.month.unwrap(),
                        to.day.unwrap(),
                    ) {
                        return Err(Error::InvalidRange {
                            from: format!(
                                "{}-{:02}-{:02}",
                                from.year.unwrap(),
                                from.month.unwrap(),
                                from.day.unwrap(),
                            ),
                            to: format!(
                                "{}-{:02}-{:02}",
                                to.year.unwrap(),
                                to.month.unwrap(),
                                to.day.unwrap(),
                            ),
                            position,
                        });
                    }
                
                    index += 1;
                
                    rules.push(Rule::Range { from, to });
                }

                Some(Token::RightBrace) => {
                    index += 1;
                    break;
                }

                _ => {
                    return Err(Error::UnexpectedToken { position: index });
                }
            }

            match tokens.get(index) {
                Some(Token::Comma) => {
                    index += 1;
                }

                Some(Token::RightBrace) => {
                    index += 1;
                    break;
                }

                _ => {
                    return Err(Error::UnexpectedToken { position: index });
                }
            }
        }

        entries.push(ConfigEntry { name, rules });
    }

    Ok(Config { entries })
}

fn parse_date(value: &str, position: usize) -> Result<Date, Error> {
    let mut parts = value.split('-');

    let year = parse_year(parts.next(), value, position)?;
    let month = parse_month(parts.next(), value, position)?;
    let day = parse_day(parts.next(), value, position)?;

    if parts.next().is_some() {
        return Err(Error::InvalidDate {
            value: value.to_owned(),
            position,
        });
    }

    if let Some(month) = month 
        && let Some(day) = day {
            let max_day = days_in_month(year, month);

            if day > max_day {
                return Err(Error::InvalidDate {
                    value: value.to_owned(),
                    position,
                });
            }
    }

    Ok(Date { year, month, day })
}

fn is_concrete(date: &Date) -> bool {
    date.year.is_some() && date.month.is_some() && date.day.is_some()
}

fn days_in_month(year: Option<u16>, month: u8) -> u8 {
    match month {
        2 => match year {
            Some(year) if is_leap_year(year) => 29,
            _ => 28,
        },
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn parse_year(value: Option<&str>, original: &str, position: usize) -> Result<Option<u16>, Error> {
    let Some(value) = value else {
        return Err(Error::InvalidDate {
            value: original.to_owned(),
            position,
        });
    };

    if value == "*" {
        return Ok(None);
    }

    if value.len() != 4 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::InvalidDate {
            value: original.to_owned(),
            position,
        });
    }

    let value = value.parse::<u16>().map_err(|_| Error::InvalidDate {
        value: original.to_owned(),
        position,
    })?;

    if value == 0 {
        return Err(Error::InvalidDate {
            value: original.to_owned(),
            position,
        });
    }

    Ok(Some(value))
}

fn parse_month(value: Option<&str>, original: &str, position: usize) -> Result<Option<u8>, Error> {
    let Some(value) = value else {
        return Err(Error::InvalidDate {
            value: original.to_owned(),
            position,
        });
    };

    if value == "*" {
        return Ok(None);
    }

    if value.len() != 2 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::InvalidDate {
            value: original.to_owned(),
            position,
        });
    }

    let value = value.parse::<u8>().map_err(|_| Error::InvalidDate {
        value: original.to_owned(),
        position,
    })?;

    if !(1..=12).contains(&value) {
        return Err(Error::InvalidDate {
            value: original.to_owned(),
            position,
        });
    }

    Ok(Some(value))
}

fn parse_day(value: Option<&str>, original: &str, position: usize) -> Result<Option<u8>, Error> {
    let Some(value) = value else {
        return Err(Error::InvalidDate {
            value: original.to_owned(),
            position,
        });
    };

    if value == "*" {
        return Ok(None);
    }

    if value.len() != 2 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::InvalidDate {
            value: original.to_owned(),
            position,
        });
    }

    let value = value.parse::<u8>().map_err(|_| Error::InvalidDate {
        value: original.to_owned(),
        position,
    })?;

    if !(1..=31).contains(&value) {
        return Err(Error::InvalidDate {
            value: original.to_owned(),
            position,
        });
    }

    Ok(Some(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lex;

   #[test]
    fn wildcard_dates() {
        assert_eq!(
            parse_date("*-03-31", 0).unwrap(),
            Date {
                year: None,
                month: Some(3),
                day: Some(31),
            }
        );
    
        assert_eq!(
            parse_date("2026-*-31", 0).unwrap(),
            Date {
                year: Some(2026),
                month: None,
                day: Some(31),
            }
        );
    
        assert_eq!(
            parse_date("*-*-31", 0).unwrap(),
            Date {
                year: None,
                month: None,
                day: Some(31),
            }
        );
    }

    #[test]
    fn wildcards_are_rejected_in_ranges() {
        let tokens = lex(
            r#"
            holidays {
                "*-03-01" .. "2026-03-31"
            }
            "#,
        )
        .unwrap();
    
        assert!(parse(&tokens).is_err());
    
        let tokens = lex(
            r#"
            holidays {
                "2026-03-01" .. "2026-*-31"
            }
            "#,
        )
        .unwrap();
    
        assert!(parse(&tokens).is_err());
    }

    #[test]
    fn reversed_range_is_rejected() {
        let tokens = lex(
            r#"
            holidays {
                "2026-06-15" .. "2026-06-10"
            }
            "#,
        )
        .unwrap();
    
        assert!(matches!(
            parse(&tokens),
            Err(Error::InvalidRange { .. })
        ));
    }
}
