use std::fs;
use std::path::Path;

use crate::{
    config,
    error::Error,
    lex,
    parse,
    tree::{Config, ConfigEntry, Date, Schedule, Time},
};

pub fn run(root: &Path, date: &Date) -> Result<Schedule, Error> {
    let source = fs::read_to_string(root.join("ring.cat")).map_err(|_| {
        Error::ScheduleNotFound {
            name: "ring.cat".to_owned(),
        }
    })?;

    let tokens = lex::lex(&source)?;
    let config = config::parse(&tokens)?;
    let entry = resolve(&config, date)?;

    load(root, entry)
}

fn load(root: &Path, entry: &ConfigEntry) -> Result<Schedule, Error> {
   	let path = root.join("bell").join(format!("{}.bell", entry.name));

    let source = fs::read_to_string(&path).map_err(|_| {
        Error::ScheduleNotFound {
            name: entry.name.clone(),
        }
    })?;

    let tokens = lex::lex(&source)?;
    parse::parse(&tokens)
}

pub fn resolve<'a>(
    config: &'a Config,
    date: &Date,
) -> Result<&'a ConfigEntry, Error> {
    let mut best: Option<(&ConfigEntry, u8)> = None;

    for entry in &config.entries {
        let priority = entry_priority(entry, date);

        let Some(priority) = priority else {
            continue;
        };

        match best {
            None => best = Some((entry, priority)),

            Some((_, best_priority)) if priority > best_priority => {
                best = Some((entry, priority));
            }

            Some((_, best_priority)) if priority == best_priority => {
                return Err(Error::AmbiguousSchedule);
            }

            Some(_) => {}
        }
    }

    best.map(|(entry, _)| entry)
        .ok_or(Error::NoSchedule)
}

fn entry_priority(entry: &ConfigEntry, date: &Date) -> Option<u8> {
    let mut priority: Option<u8> = None;

    for rule in &entry.rules {
        let current = match rule {
            crate::tree::Rule::Default => 1,

            crate::tree::Rule::Automatic => {
                continue;
            }

            crate::tree::Rule::Always => 2,

            crate::tree::Rule::Range { from, to } => {
                if date_in_range(date, from, to) {
                    4
                } else {
                    continue;
                }
            }
        };

        priority = Some(priority.map_or(current, |old| old.max(current)));
    }

    priority
}

fn date_in_range(date: &Date, from: &Date, to: &Date) -> bool {
    let Some(year) = date.year else {
        return false;
    };

    let Some(month) = date.month else {
        return false;
    };

    let Some(day) = date.day else {
        return false;
    };

    let Some(from_year) = from.year else {
        return false;
    };

    let Some(from_month) = from.month else {
        return false;
    };

    let Some(from_day) = from.day else {
        return false;
    };

    let Some(to_year) = to.year else {
        return false;
    };

    let Some(to_month) = to.month else {
        return false;
    };

    let Some(to_day) = to.day else {
        return false;
    };

    let current = (year, month, day);
    let start = (from_year, from_month, from_day);
    let end = (to_year, to_month, to_day);

    current >= start && current <= end
}

pub struct Clock {
    schedule: Schedule,
    next: usize,
}

impl Clock {
    pub fn new(schedule: Schedule, time: &Time) -> Self {
        let next = schedule
            .times
            .iter()
            .position(|ring| {
                (ring.hour, ring.minute) >= (time.hour, time.minute)
            })
            .unwrap_or(schedule.times.len());

        Self { schedule, next }
    }

    pub fn tick(&mut self, time: &Time) -> bool {
        let Some(next) = self.schedule.times.get(self.next) else {
            return false;
        };

        if next.hour == time.hour && next.minute == time.minute {
            self.next += 1;
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lex;

    #[test]
    fn run_loads_active_schedule() {
        let root = std::env::temp_dir().join("bellc-test");
    
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("bell")).unwrap();
    
        std::fs::write(
            root.join("ring.cat"),
            r#"
            standard {
                default
            }
    
            holidays {
                "2026-06-10" .. "2026-06-15"
            }
            "#,
        )
        .unwrap();
    
        std::fs::write(
            root.join("bell/standard.bell"),
            r#""08:00", "09:00""#,
        )
        .unwrap();
    
        std::fs::write(
            root.join("bell/holidays.bell"),
            r#""10:00", "11:00""#,
        )
        .unwrap();
    
        let schedule = run(&root, &Date {
            year: Some(2026),
            month: Some(6),
            day: Some(12),
        })
        .unwrap();
    
        assert_eq!(schedule.times.len(), 2);
        assert_eq!(schedule.times[0].hour, 10);
        assert_eq!(schedule.times[1].hour, 11);
    
        let _ = std::fs::remove_dir_all(root);
    }

    fn date(year: u16, month: u8, day: u8) -> Date {
        Date {
            year: Some(year),
            month: Some(month),
            day: Some(day),
        }
    }

    fn config(source: &str) -> Config {
        let tokens = lex(source).unwrap();
        config::parse(&tokens).unwrap()
    }

    #[test]
    fn clock_rings() {
        let schedule = Schedule {
            times: vec![
                Time { hour: 8, minute: 0 },
                Time { hour: 9, minute: 0 },
            ],
        };
    
        let mut clock = Clock::new(schedule, &Time { hour: 7, minute: 0 });
    
        assert!(clock.tick(&Time { hour: 8, minute: 0 }));
        assert!(!clock.tick(&Time { hour: 8, minute: 0 }));
        assert!(!clock.tick(&Time { hour: 8, minute: 30 }));
        assert!(clock.tick(&Time { hour: 9, minute: 0 }));
    }
    
    #[test]
    fn clock_stops_after_last() {
        let schedule = Schedule {
            times: vec![Time { hour: 8, minute: 0 }],
        };
    
        let mut clock = Clock::new(schedule, &Time { hour: 7, minute: 0 });
    
        assert!(clock.tick(&Time { hour: 8, minute: 0 }));
        assert!(!clock.tick(&Time { hour: 9, minute: 0 }));
    }

    #[test]
    fn default_schedule() {
        let config = config(
            r#"
            standard {
                default
            }
            "#,
        );

        let entry = resolve(&config, &date(2026, 6, 10)).unwrap();

        assert_eq!(entry.name, "standard");
    }

    #[test]
    fn always_schedule() {
        let config = config(
            r#"
            standard {
                default
            }

            custom {
                always
            }
            "#,
        );

        let entry = resolve(&config, &date(2026, 6, 10)).unwrap();

        assert_eq!(entry.name, "custom");
    }

    #[test]
    fn range_overrides_always() {
        let config = config(
            r#"
            standard {
                default
            }

            custom {
                always
            }

            holidays {
                "2026-06-10" .. "2026-06-15"
            }
            "#,
        );

        let entry = resolve(&config, &date(2026, 6, 12)).unwrap();

        assert_eq!(entry.name, "holidays");
    }

    #[test]
    fn range_only_matches_inside_range() {
        let config = config(
            r#"
            standard {
                default
            }

            holidays {
                "2026-06-10" .. "2026-06-15"
            }
            "#,
        );

        let entry = resolve(&config, &date(2026, 6, 20)).unwrap();

        assert_eq!(entry.name, "standard");
    }

    #[test]
    fn overlapping_ranges_are_ambiguous() {
        let config = config(
            r#"
            first {
                "2026-06-10" .. "2026-06-15"
            }

            second {
                "2026-06-12" .. "2026-06-20"
            }
            "#,
        );

        assert!(matches!(
            resolve(&config, &date(2026, 6, 13)),
            Err(Error::AmbiguousSchedule)
        ));
    }

    #[test]
    fn no_default_or_matching_schedule() {
        let config = config(
            r#"
            holidays {
                "2026-06-10" .. "2026-06-15"
            }
            "#,
        );

        assert!(matches!(
            resolve(&config, &date(2026, 6, 20)),
            Err(Error::NoSchedule)
        ));
    }

    #[test]
    fn clock_starts_before_ring() {
        let schedule = Schedule {
            times: vec![
                Time { hour: 8, minute: 0 },
                Time { hour: 9, minute: 0 },
            ],
        };
    
        let mut clock = Clock::new(
            schedule,
            &Time { hour: 7, minute: 30 },
        );
    
        assert!(clock.tick(&Time { hour: 8, minute: 0 }));
    }
    
    #[test]
    fn clock_starts_between_rings() {
        let schedule = Schedule {
            times: vec![
                Time { hour: 8, minute: 0 },
                Time { hour: 9, minute: 0 },
            ],
        };
    
        let mut clock = Clock::new(
            schedule,
            &Time { hour: 8, minute: 30 },
        );
    
        assert!(!clock.tick(&Time { hour: 8, minute: 30 }));
        assert!(clock.tick(&Time { hour: 9, minute: 0 }));
    }
    
    #[test]
    fn clock_starts_after_last_ring() {
        let schedule = Schedule {
            times: vec![
                Time { hour: 8, minute: 0 },
                Time { hour: 9, minute: 0 },
            ],
        };
    
        let mut clock = Clock::new(
            schedule,
            &Time { hour: 10, minute: 0 },
        );
    
        assert!(!clock.tick(&Time { hour: 10, minute: 0 }));
    }

    #[test]
    fn range_crosses_year() {
        let config = config(
            r#"
            holiday {
                "2026-12-20" .. "2027-01-10"
            }
            "#,
        );
    
        assert_eq!(
            resolve(&config, &date(2026, 12, 25)).unwrap().name,
            "holiday"
        );
    
        assert_eq!(
            resolve(&config, &date(2027, 1, 5)).unwrap().name,
            "holiday"
        );
    
        assert!(matches!(
            resolve(&config, &date(2027, 1, 15)),
            Err(Error::NoSchedule)
        ));
    }

    #[test]
    fn range_crosses_month() {
        let config = config(
            r#"
            holiday {
                "2026-05-25" .. "2026-06-05"
            }
            "#,
        );
    
        assert_eq!(
            resolve(&config, &date(2026, 5, 30)).unwrap().name,
            "holiday"
        );
    
        assert_eq!(
            resolve(&config, &date(2026, 6, 3)).unwrap().name,
            "holiday"
        );
    
        assert!(matches!(
            resolve(&config, &date(2026, 6, 10)),
            Err(Error::NoSchedule)
        ));
    }
}
