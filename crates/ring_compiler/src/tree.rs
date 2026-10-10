use std::fmt;

use crate::error::Location;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
}

/// An empty `times` list is a schedule that rings nothing (`Disable`).
#[derive(Debug, PartialEq, Eq)]
pub struct Schedule {
    pub times: Vec<Time>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub entries: Vec<ConfigEntry>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConfigEntry {
    pub name: String,
    pub rules: Vec<Rule>,
    pub at: Location,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Rule {
    Date(Date),
    Range { from: Date, to: Date },
    Weekday(Weekday),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// A checked project: the calendar rules and one schedule per block, in block order.
#[derive(Debug)]
pub struct Project {
    pub config: Config,
    pub schedules: Vec<Schedule>,
}

impl Weekday {
    pub fn from_name(name: &str) -> Option<Weekday> {
        match name {
            "Monday" => Some(Self::Monday),
            "Tuesday" => Some(Self::Tuesday),
            "Wednesday" => Some(Self::Wednesday),
            "Thursday" => Some(Self::Thursday),
            "Friday" => Some(Self::Friday),
            "Saturday" => Some(Self::Saturday),
            "Sunday" => Some(Self::Sunday),
            _ => None,
        }
    }
}

impl Date {
    pub fn weekday(&self) -> Weekday {
        let year = i64::from(self.year) - i64::from(self.month <= 2);
        let month = i64::from(self.month);
        let day = i64::from(self.day);

        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        let days = era * 146097 + day_of_era - 719468;

        // 1970/01/01 was a Thursday :P
        match (days + 4).rem_euclid(7) {
            0 => Weekday::Sunday,
            1 => Weekday::Monday,
            2 => Weekday::Tuesday,
            3 => Weekday::Wednesday,
            4 => Weekday::Thursday,
            5 => Weekday::Friday,
            _ => Weekday::Saturday,
        }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}-{}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: u16, month: u8, day: u8) -> Date {
        Date { year, month, day }
    }

    #[test]
    fn weekdays() {
        assert_eq!(date(1970, 1, 1).weekday(), Weekday::Thursday);
        assert_eq!(date(2000, 2, 29).weekday(), Weekday::Tuesday);
        assert_eq!(date(2026, 9, 7).weekday(), Weekday::Monday);
        assert_eq!(date(2026, 10, 10).weekday(), Weekday::Saturday);
        assert_eq!(date(2026, 10, 31).weekday(), Weekday::Saturday);
        assert_eq!(date(2027, 1, 3).weekday(), Weekday::Sunday);
    }

    #[test]
    fn dates_order_chronologically() {
        assert!(date(2026, 12, 31) < date(2027, 1, 1));
        assert!(date(2026, 9, 7) < date(2026, 10, 1));
        assert!(date(2026, 9, 7) < date(2026, 9, 8));
    }
}
