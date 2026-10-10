use std::{fs, io, path::Path};

use crate::{
    config,
    error::{Error, Failure},
    lex, parse,
    tree::{Config, Date, Project, Rule, Schedule, Time},
};

fn fail(file: &Path, error: Error) -> Failure {
    Failure {
        file: file.to_path_buf(),
        error,
    }
}

/// Reads `ring.schedule` and the `schedule/<name>.time` file of every block.
///
/// The file name is the lowercased block name plus `.time`. A `.time` file that no block
/// names is never read.
pub fn load_project(root: &Path) -> Result<Project, Failure> {
    let ring = root.join("ring.schedule");

    let source = fs::read_to_string(&ring).map_err(|error| {
        fail(
            &ring,
            Error::Read {
                message: error.to_string(),
            },
        )
    })?;

    let tokens = lex::lex(&source).map_err(|error| fail(&ring, error))?;
    let config = config::parse(&tokens).map_err(|error| fail(&ring, error))?;

    let mut schedules = Vec::with_capacity(config.entries.len());

    for entry in &config.entries {
        let file_name = format!("{}.time", entry.name.to_ascii_lowercase());
        let path = root.join("schedule").join(&file_name);

        let source = match fs::read_to_string(&path) {
            Ok(source) => source,

            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(fail(
                    &ring,
                    Error::ScheduleNotFound {
                        path: format!("schedule/{file_name}"),
                        at: entry.at,
                    },
                ));
            }

            Err(error) => {
                return Err(fail(
                    &path,
                    Error::Read {
                        message: error.to_string(),
                    },
                ));
            }
        };

        let tokens = lex::lex(&source).map_err(|error| fail(&path, error))?;
        let schedule = parse::parse(&tokens).map_err(|error| fail(&path, error))?;

        schedules.push(schedule);
    }

    Ok(Project { config, schedules })
}

/// Index of the block that applies on `date`. Later blocks override earlier ones.
/// `None` means no block matches, which rings nothing.
pub fn resolve(config: &Config, date: &Date) -> Option<usize> {
    config
        .entries
        .iter()
        .rposition(|entry| entry.rules.iter().any(|rule| matches(rule, date)))
}

#[allow(dead_code)]
pub fn schedule_on<'a>(project: &'a Project, date: &Date) -> Option<&'a Schedule> {
    resolve(&project.config, date).map(|index| &project.schedules[index])
}

fn matches(rule: &Rule, date: &Date) -> bool {
    match rule {
        Rule::Date(day) => day == date,
        Rule::Range { from, to } => from <= date && date <= to,
        Rule::Weekday(weekday) => date.weekday() == *weekday,
    }
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
            .position(|ring| (ring.hour, ring.minute) >= (time.hour, time.minute))
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

    fn date(year: u16, month: u8, day: u8) -> Date {
        Date { year, month, day }
    }

    fn config(source: &str) -> Config {
        let tokens = lex(source).unwrap();
        config::parse(&tokens).unwrap()
    }

    fn name_on(config: &Config, date: &Date) -> Option<String> {
        resolve(config, date).map(|index| config.entries[index].name.clone())
    }

    /// The project `ringc init` generates.
    fn template() -> Config {
        config(include_str!("../templates/ring.schedule"))
    }

    fn project_dir(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("ringc-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("schedule")).unwrap();
        root
    }

    #[test]
    fn later_block_overrides_earlier() {
        let config = config(
            "
            Standard { 2026-9-7 .. 2027-6-26 }
            Ramadan { 2027-2-8 .. 2027-3-9 }
            Holidays { Sunday 2026-10-31 }
            ",
        );

        // Monday in the school year
        assert_eq!(
            name_on(&config, &date(2026, 9, 7)).as_deref(),
            Some("Standard")
        );
        // Monday in Ramadan
        assert_eq!(
            name_on(&config, &date(2027, 2, 15)).as_deref(),
            Some("Ramadan")
        );
        // Sunday in the school year
        assert_eq!(
            name_on(&config, &date(2026, 9, 13)).as_deref(),
            Some("Holidays")
        );
        // Sunday in Ramadan
        assert_eq!(
            name_on(&config, &date(2027, 2, 14)).as_deref(),
            Some("Holidays")
        );
        // A named date
        assert_eq!(
            name_on(&config, &date(2026, 10, 31)).as_deref(),
            Some("Holidays")
        );
    }

    #[test]
    fn block_order_is_precedence() {
        let config = config(
            "
            Holidays { Sunday }
            Standard { 2026-9-7 .. 2027-6-26 }
            ",
        );

        // Same Sunday as above, but now Standard is later and wins.
        assert_eq!(
            name_on(&config, &date(2026, 9, 13)).as_deref(),
            Some("Standard")
        );
    }

    #[test]
    fn range_is_inclusive() {
        let config = config("Break { 2026-12-20 .. 2027-1-10 }");

        assert_eq!(name_on(&config, &date(2026, 12, 19)), None);
        assert_eq!(
            name_on(&config, &date(2026, 12, 20)).as_deref(),
            Some("Break")
        );
        assert_eq!(
            name_on(&config, &date(2026, 12, 31)).as_deref(),
            Some("Break")
        );
        assert_eq!(
            name_on(&config, &date(2027, 1, 10)).as_deref(),
            Some("Break")
        );
        assert_eq!(name_on(&config, &date(2027, 1, 11)), None);
    }

    #[test]
    fn unmatched_day_is_none() {
        let config = config("Standard { 2026-9-7 .. 2027-6-26 }");

        assert_eq!(resolve(&config, &date(2026, 7, 15)), None);
    }

    #[test]
    fn init_template_resolves_as_intended() {
        let config = template();

        let on = |y, m, d| name_on(&config, &date(y, m, d));

        assert_eq!(on(2026, 9, 7).as_deref(), Some("Standard"));
        assert_eq!(on(2026, 9, 13).as_deref(), Some("Holidays"));
        assert_eq!(on(2026, 10, 31).as_deref(), Some("Holidays"));
        assert_eq!(on(2027, 2, 9).as_deref(), Some("Ramadan"));
        assert_eq!(on(2027, 3, 9).as_deref(), Some("Holidays"));
        assert_eq!(on(2026, 7, 1), None);
    }

    #[test]
    fn init_template_passes_check() {
        let root = project_dir("template");

        fs::write(
            root.join("ring.schedule"),
            include_str!("../templates/ring.schedule"),
        )
        .unwrap();
        fs::write(
            root.join("schedule/standard.time"),
            include_str!("../templates/schedule/standard.time"),
        )
        .unwrap();
        fs::write(
            root.join("schedule/ramadan.time"),
            include_str!("../templates/schedule/ramadan.time"),
        )
        .unwrap();
        fs::write(
            root.join("schedule/holidays.time"),
            include_str!("../templates/schedule/holidays.time"),
        )
        .unwrap();

        let project = load_project(&root).unwrap();

        assert_eq!(project.schedules.len(), 3);
        assert_eq!(project.schedules[0].times.len(), 12);
        assert_eq!(project.schedules[1].times.len(), 12);
        assert_eq!(project.schedules[2].times.len(), 0);

        let school_day = schedule_on(&project, &date(2026, 9, 7)).unwrap();
        assert_eq!(school_day.times[0], Time { hour: 8, minute: 0 });

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_schedule_file_points_at_the_block() {
        let root = project_dir("missing");

        fs::write(root.join("ring.schedule"), "Standard {\n    Monday\n}\n").unwrap();

        let failure = load_project(&root).unwrap_err();

        assert!(failure.file.ends_with("ring.schedule"));
        assert!(matches!(
            failure.error,
            Error::ScheduleNotFound {
                at: crate::error::Location { line: 1, column: 1 },
                ..
            }
        ));
        assert!(failure.to_string().contains("ring.schedule:1:1"));
        assert!(failure.to_string().contains("schedule/standard.time"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn error_in_a_time_file_names_that_file() {
        let root = project_dir("bad-time");

        fs::write(root.join("ring.schedule"), "Standard { Monday }").unwrap();
        fs::write(root.join("schedule/standard.time"), "08:00\n25:00\n").unwrap();

        let failure = load_project(&root).unwrap_err();

        assert!(failure.file.ends_with("schedule/standard.time"));
        assert!(failure.to_string().contains("standard.time:2:1"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn unreferenced_time_file_is_ignored() {
        let root = project_dir("unreferenced");

        fs::write(root.join("ring.schedule"), "Standard { Monday }").unwrap();
        fs::write(root.join("schedule/standard.time"), "08:00").unwrap();
        fs::write(
            root.join("schedule/exam.time"),
            "this is not valid at all {",
        )
        .unwrap();

        assert!(load_project(&root).is_ok());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_ring_schedule_is_a_read_error() {
        let root = project_dir("no-ring");

        let failure = load_project(&root).unwrap_err();

        assert!(matches!(failure.error, Error::Read { .. }));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clock_rings() {
        let schedule = Schedule {
            times: vec![Time { hour: 8, minute: 0 }, Time { hour: 9, minute: 0 }],
        };

        let mut clock = Clock::new(schedule, &Time { hour: 7, minute: 0 });

        assert!(clock.tick(&Time { hour: 8, minute: 0 }));
        assert!(!clock.tick(&Time { hour: 8, minute: 0 }));
        assert!(!clock.tick(&Time {
            hour: 8,
            minute: 30
        }));
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
    fn clock_starts_before_ring() {
        let schedule = Schedule {
            times: vec![Time { hour: 8, minute: 0 }, Time { hour: 9, minute: 0 }],
        };

        let mut clock = Clock::new(
            schedule,
            &Time {
                hour: 7,
                minute: 30,
            },
        );

        assert!(clock.tick(&Time { hour: 8, minute: 0 }));
    }

    #[test]
    fn clock_starts_between_rings() {
        let schedule = Schedule {
            times: vec![Time { hour: 8, minute: 0 }, Time { hour: 9, minute: 0 }],
        };

        let mut clock = Clock::new(
            schedule,
            &Time {
                hour: 8,
                minute: 30,
            },
        );

        assert!(!clock.tick(&Time {
            hour: 8,
            minute: 30
        }));
        assert!(clock.tick(&Time { hour: 9, minute: 0 }));
    }

    #[test]
    fn clock_starts_after_last_ring() {
        let schedule = Schedule {
            times: vec![Time { hour: 8, minute: 0 }, Time { hour: 9, minute: 0 }],
        };

        let mut clock = Clock::new(
            schedule,
            &Time {
                hour: 10,
                minute: 0,
            },
        );

        assert!(!clock.tick(&Time {
            hour: 10,
            minute: 0
        }));
    }
}
