use crate::{
    error::{Error, Location},
    lex::{Kind, Token},
    tree::{Config, ConfigEntry, Date, Rule, Weekday},
};

pub fn parse(tokens: &[Token]) -> Result<Config, Error> {
    let mut entries: Vec<ConfigEntry> = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        let token = &tokens[index];

        let name = match &token.kind {
            Kind::Word(name) if is_identifier(name) => name.clone(),
            _ => return Err(Error::UnexpectedToken { at: token.at }),
        };

        let at = token.at;

        if entries
            .iter()
            .any(|entry| entry.name.eq_ignore_ascii_case(&name))
        {
            return Err(Error::DuplicateSchedule { name, at });
        }

        index += 1;

        match tokens.get(index) {
            Some(Token {
                kind: Kind::LeftBrace,
                ..
            }) => index += 1,
            Some(token) => return Err(Error::UnexpectedToken { at: token.at }),
            None => return Err(unexpected_end(tokens)),
        }

        let mut rules = Vec::new();

        loop {
            let Some(token) = tokens.get(index) else {
                return Err(unexpected_end(tokens));
            };

            match &token.kind {
                Kind::RightBrace => {
                    index += 1;
                    break;
                }

                Kind::Word(word) if word.starts_with(|c: char| c.is_ascii_alphabetic()) => {
                    let Some(weekday) = Weekday::from_name(word) else {
                        return Err(Error::UnexpectedToken { at: token.at });
                    };

                    rules.push(Rule::Weekday(weekday));
                    index += 1;
                }

                Kind::Word(word) => {
                    let from = parse_date(word, token.at)?;
                    index += 1;

                    if matches!(
                        tokens.get(index),
                        Some(Token {
                            kind: Kind::Range,
                            ..
                        })
                    ) {
                        index += 1;

                        let Some(end) = tokens.get(index) else {
                            return Err(unexpected_end(tokens));
                        };

                        let Kind::Word(word) = &end.kind else {
                            return Err(Error::UnexpectedToken { at: end.at });
                        };

                        let to = parse_date(word, end.at)?;

                        if from > to {
                            return Err(Error::InvalidRange {
                                from: from.to_string(),
                                to: to.to_string(),
                                at: token.at,
                            });
                        }

                        index += 1;
                        rules.push(Rule::Range { from, to });
                    } else {
                        rules.push(Rule::Date(from));
                    }
                }

                _ => return Err(Error::UnexpectedToken { at: token.at }),
            }
        }

        entries.push(ConfigEntry { name, rules, at });
    }

    Ok(Config { entries })
}

fn unexpected_end(tokens: &[Token]) -> Error {
    Error::UnexpectedEnd {
        at: tokens
            .last()
            .map(|token| token.at)
            .unwrap_or(Location { line: 1, column: 1 }),
    }
}

fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();

    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// `YYYY-M-D`. The year has four digits, month and day have one or two.
fn parse_date(value: &str, at: Location) -> Result<Date, Error> {
    let invalid = || Error::InvalidDate {
        value: value.to_owned(),
        at,
    };

    let mut parts = value.split('-');

    let (Some(year), Some(month), Some(day), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(invalid());
    };

    if year.len() != 4 {
        return Err(invalid());
    }

    let year = digits(year, 4).ok_or_else(invalid)?;
    let month = digits(month, 2).ok_or_else(invalid)?;
    let day = digits(day, 2).ok_or_else(invalid)?;

    if year == 0 || !(1..=12).contains(&month) {
        return Err(invalid());
    }

    let month = month as u8;

    if day == 0 || day > u16::from(days_in_month(year, month)) {
        return Err(invalid());
    }

    Ok(Date {
        year,
        month,
        day: day as u8,
    })
}

fn digits(text: &str, max_len: usize) -> Option<u16> {
    if text.is_empty() || text.len() > max_len || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    text.parse().ok()
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lex;

    const HERE: Location = Location { line: 1, column: 1 };

    fn config(source: &str) -> Result<Config, Error> {
        parse(&lex(source).unwrap())
    }

    fn date(year: u16, month: u8, day: u8) -> Date {
        Date { year, month, day }
    }

    #[test]
    fn dates_without_leading_zeroes() {
        assert_eq!(parse_date("2026-9-7", HERE).unwrap(), date(2026, 9, 7));
        assert_eq!(parse_date("2026-09-07", HERE).unwrap(), date(2026, 9, 7));
        assert_eq!(parse_date("2027-1-14", HERE).unwrap(), date(2027, 1, 14));
    }

    #[test]
    fn invalid_dates() {
        for value in [
            "2026-2-30",
            "2027-2-29",
            "2026-13-1",
            "2026-0-1",
            "2026-1-0",
            "2026-4-31",
            "26-9-7",
            "2026-9",
            "2026-9-7-1",
            "2026-9-",
            "0000-1-1",
            "2026-009-7",
        ] {
            assert!(
                matches!(parse_date(value, HERE), Err(Error::InvalidDate { .. })),
                "{value} should be rejected"
            );
        }
    }

    #[test]
    fn leap_day() {
        assert_eq!(parse_date("2028-2-29", HERE).unwrap(), date(2028, 2, 29));
        assert_eq!(parse_date("2000-2-29", HERE).unwrap(), date(2000, 2, 29));
        assert!(parse_date("1900-2-29", HERE).is_err());
    }

    #[test]
    fn block_with_every_rule_kind() {
        let config = config(
            "
            Holidays {
                // Weekend
                Sunday

                2026-10-31
                2026-11-6
                2026-12-6 .. 2026-12-13
            }
            ",
        )
        .unwrap();

        assert_eq!(config.entries.len(), 1);
        assert_eq!(config.entries[0].name, "Holidays");
        assert_eq!(
            config.entries[0].rules,
            vec![
                Rule::Weekday(Weekday::Sunday),
                Rule::Date(date(2026, 10, 31)),
                Rule::Date(date(2026, 11, 6)),
                Rule::Range {
                    from: date(2026, 12, 6),
                    to: date(2026, 12, 13)
                },
            ]
        );
    }

    #[test]
    fn blocks_keep_file_order() {
        let config = config("B { Monday }\nA { Tuesday }\nC { }").unwrap();

        let names: Vec<_> = config.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["B", "A", "C"]);
    }

    #[test]
    fn range_crosses_month_and_year() {
        let config = config("Break { 2026-12-20 .. 2027-1-10 }").unwrap();

        assert_eq!(
            config.entries[0].rules,
            vec![Rule::Range {
                from: date(2026, 12, 20),
                to: date(2027, 1, 10)
            }]
        );
    }

    #[test]
    fn single_day_range_is_allowed() {
        assert!(config("Day { 2026-6-10 .. 2026-6-10 }").is_ok());
    }

    #[test]
    fn reversed_range_is_rejected() {
        assert!(matches!(
            config("Holidays {\n    2026-6-15 .. 2026-6-10\n}"),
            Err(Error::InvalidRange {
                at: Location { line: 2, column: 5 },
                ..
            })
        ));
    }

    #[test]
    fn commas_are_rejected() {
        assert!(matches!(
            lex("Holidays { Sunday, 2026-10-31 }"),
            Err(Error::UnexpectedCharacter { character: ',', .. })
        ));
    }

    #[test]
    fn duplicate_names_are_rejected_ignoring_case() {
        assert!(matches!(
            config("Standard { Monday }\nstandard { Tuesday }"),
            Err(Error::DuplicateSchedule {
                at: Location { line: 2, column: 1 },
                ..
            })
        ));

        assert!(matches!(
            config("Standard { Monday }\nStandard { Tuesday }"),
            Err(Error::DuplicateSchedule { .. })
        ));
    }

    #[test]
    fn unknown_weekday_is_rejected() {
        assert!(matches!(
            config("Holidays { Funday }"),
            Err(Error::UnexpectedToken { .. })
        ));
        assert!(matches!(
            config("Holidays { sunday }"),
            Err(Error::UnexpectedToken { .. })
        ));
    }

    #[test]
    fn structure_errors() {
        assert!(matches!(
            config("Standard { 2026-9-7"),
            Err(Error::UnexpectedEnd { .. })
        ));
        assert!(matches!(
            config("Standard"),
            Err(Error::UnexpectedEnd { .. })
        ));
        assert!(matches!(
            config("Standard 2026-9-7"),
            Err(Error::UnexpectedToken { .. })
        ));
        assert!(matches!(
            config("2026-9-7 { Sunday }"),
            Err(Error::UnexpectedToken { .. })
        ));
        assert!(matches!(
            config("Standard { 2026-9-7 .. }"),
            Err(Error::UnexpectedToken { .. })
        ));
        assert!(matches!(
            config("Standard { .. 2026-9-7 }"),
            Err(Error::UnexpectedToken { .. })
        ));
    }

    #[test]
    fn invalid_date_reports_its_location() {
        assert!(matches!(
            config("Standard {\n    2026-2-30\n}"),
            Err(Error::InvalidDate {
                at: Location { line: 2, column: 5 },
                ..
            })
        ));
    }
}
