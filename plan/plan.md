# ringc

## 1. Purpose

`ringc` is a compiler and deployment tool for a school bell scheduling system.

It lets a school administrator define the academic calendar and bell timetable in human-editable files, validate the project, compile it into a device-readable schedule, install the system, and update the schedule without reinstalling the firmware.

The project separates two concerns:

```text
ring.schedule
    defines WHEN a schedule is active

*.time
    defines WHAT that schedule does
```

The embedded device executes the compiled schedule using its real-time clock and controls the physical bell.

```text
administrator
    ↓
source project
    ↓
ringc
    ↓
compiled schedule
    ↓
bell device
    ↓
RTC + schedule + controls
    ↓
physical bell
```

## 2. Project layout

The initial project template is:

```text
ringc-project/
├── ring.schedule
└── schedule/
    ├── standard.time
    ├── ramadan.time
    └── holidays.time
```

Additional schedules, such as `exam.time`, can be added when needed.

`ring.schedule` defines the calendar rules for each named schedule. The files in `schedule/` define the corresponding ringing times or disable automatic ringing.

## 3. Administrative lifecycle

Initial setup:

```text
ringc init
    ↓
project template
    ↓
administrator edits files
    ↓
ringc check <path>
    ↓
ringc build <path>
    ↓
ringc install
```

Routine schedule changes:

```text
administrator edits files
    ↓
ringc check <path>
    ↓
ringc build <path>
    ↓
ringc update
```

`install` installs the framework or firmware together with the initial schedule.

`update` replaces the schedule on an already-installed device without reinstalling the framework or firmware.

Timetable changes should remain data updates, not firmware rebuilds.

## 4. `ringc init`

`init` creates a prebuilt project template containing the calendar configuration and example schedules.

The template should include:

- `standard.time` for the normal school timetable.
- `ramadan.time` for the Ramadan timetable.
- `holidays.time` for days when automatic ringing is disabled.

Additional examples, such as an examination timetable, may be included later.

`init` must not overwrite an existing project.

## 5. `ring.schedule`

`ring.schedule` is the calendar and schedule-selection configuration.

Its current syntax uses named blocks:

```text
Standard {
    // School year, all levels
    2026-9-7 .. 2027-6-26
}

Ramadan {
    // Estimate
    2027-2-8 .. 2027-3-9
}

Holidays {
    // Weekend
    Sunday

    // Unity Day
    2026-10-31,

    // Green March
    2026-11-6
}
```

Each block identifies a schedule and contains the rules that select it.

The corresponding schedule files are stored in the `schedule/` directory:

```text
Standard  → standard.time
Ramadan   → ramadan.time
Holidays  → holidays.time
```

The exact identifier-to-filename normalization must be consistent throughout the compiler.

The calendar answers which schedule applies on a given date. It does not contain the ringing times themselves.

## 6. Date syntax

Dates use the following form:

```text
YYYY-M-D
```

Month and day values do not require leading zeroes.

Examples:

```text
2026-9-7
2026-10-31
2027-1-14
```

A date range uses `..`:

```text
2026-9-7 .. 2027-6-26
```

Ranges are inclusive. Both endpoints are included.

Ranges can cross month and year boundaries:

```text
2026-12-20 .. 2027-1-10
```

Dates must be valid calendar dates. Invalid dates, such as February 31, must be rejected.

Ranges must have valid, ordered endpoints. A reversed range must produce an explicit error rather than being silently corrected.

The current template uses concrete dates rather than the wildcard syntax described in earlier designs.

## 7. Weekday rules

Weekday names represent recurring days of the week.

The current syntax includes:

```text
Holidays {
    Sunday
}
```

This selects the `Holidays` schedule on Sundays.

The supported weekday names are:

```text
Monday
Tuesday
Wednesday
Thursday
Friday
Saturday
Sunday
```

Weekday rules and explicit date rules can appear in the same block.

For example:

```text
Holidays {
    Sunday,

    2026-11-6,
    2026-11-18,

    2026-12-6 .. 2026-12-13
}
```

This combines a recurring weekday with individual dates and date ranges.

The resolver must define how overlapping rules from different schedule blocks are handled. It must not silently choose between conflicting schedules.

## 8. `.time` files

A `.time` file defines the behavior of a selected schedule.

A normal schedule contains an ordered list of ringing times:

```text
// Morning / Sba7
"08:00", "09:00",
"09:55", "10:05",
"11:00", "12:00",

// Afternoon / L3chia
"14:00", "15:00",
"15:55", "16:05",
"17:00", "18:00"
```

Comments organize the timetable for administrators. They do not change its execution.

Newlines and comments are formatting. The schedule is one list of times.

### Time syntax

Hours may contain one or two digits. Minutes must contain exactly two digits.

Both forms are valid:

```text
"8:00"
"08:00"
```

The valid time range is `00:00` through `23:59`.

Invalid examples include:

```text
"8:0"
"08:0"
"800"
"8:000"
"25:00"
"08:60"
```

A trailing comma is permitted. Missing or repeated separators must be rejected.

## 9. Disabled schedules

A schedule file can contain:

```text
Disable
```

For example, `schedule/holidays.time` contains:

```text
Disable
```

When selected, this schedule suppresses automatic ringing for that date.

The calendar rules remain in `ring.schedule`. The disabled state belongs to the schedule file.

This is distinct from the physical disable control on the device, which temporarily suppresses automatic ringing regardless of the configured timetable.

## 10. Calendar and schedule resolution

The resolver receives the parsed calendar configuration and the current date. It determines which schedule applies.

Rules can include:

- Individual dates.
- Inclusive date ranges.
- Recurring weekdays.
- Multiple rules within a schedule block.

The resolver must account for overlapping matches. For example, a date may match both a weekday rule and a date range belonging to different schedules.

The selection policy must be explicitly defined. When multiple schedules match and no precedence rule resolves the conflict, the compiler must report an ambiguity rather than guess.

If no schedule applies, the resolver must report that no schedule was found.

## 11. Compiler pipeline

The host-side compiler follows this pipeline:

```text
source files
    ↓
lexing
    ↓
parsing
    ↓
typed syntax structures
    ↓
semantic validation
    ↓
calendar resolution
    ↓
compiled schedule
```

The source modules are organized around these responsibilities:

```text
lex.rs
    lexical analysis

parse.rs
    .time parsing

config.rs
    ring.schedule parsing and validation

tree.rs
    typed syntax structures

schedule.rs
    calendar resolution and clock logic

error.rs
    compiler and runtime errors

cli.rs
    command dispatch

main.rs
    program entry point
```

`main.rs` remains minimal and delegates to `cli::run()`.

## 12. `ringc check <path>`

`check` validates a project without installing or modifying a device.

It verifies the project structure, reads `ring.schedule`, parses its rules, resolves schedule references, and validates the required `.time` files.

Conceptually:

```text
project path
    ↓
ring.schedule exists
    ↓
lex and parse configuration
    ↓
validate calendar rules
    ↓
resolve schedule references
    ↓
find required .time files
    ↓
lex and parse schedules
    ↓
semantic validation
```

An unrelated `.time` file should not automatically cause an error simply because it exists. Validation should follow the project's actual references and selection requirements.

`check` answers whether the project is valid enough to build.

## 13. `ringc build <path>`

`build` converts a validated source project into a compact representation that the embedded device can consume.

```text
ring.schedule
schedule/*.time
    ↓
ringc build
    ↓
compiled schedule
```

The compiled representation contains the information needed for date-based schedule selection and ringing-time execution.

Its exact binary format remains a separate implementation decision.

The output is schedule data, not a new firmware image.

## 14. `ringc install`

`install` performs the initial device setup.

It installs the required firmware or framework and the compiled schedule.

```text
built project
    ↓
ringc install
    ↓
device
├── firmware / framework
└── schedule
```

The device should be usable after a successful installation.

## 15. `ringc update`

`update` replaces the schedule on an already-installed device.

```text
updated source project
    ↓
check
    ↓
build
    ↓
ringc update
    ↓
existing device
    ↓
updated schedule
```

A change to ringing times or calendar rules must not require reinstalling the firmware.

## 16. Device architecture

The device separates stable software from replaceable schedule data:

```text
DEVICE
├── firmware / framework
└── schedule
```

The firmware implements generic bell-controller behavior. The schedule contains school-specific calendar and timetable data.

This separation keeps routine administrative changes independent of firmware maintenance.

## 17. Runtime

The device obtains the current date and time from its real-time clock (RTC).

The date determines the active schedule. The time determines when the next ringing event should occur.

```text
RTC
 ↓
current date
 ↓
active schedule
 ↓
current time
 ↓
clock
 ↓
ringing event?
 ├── no  → continue
 └── yes → activate bell
```

The device must not reparse the source language or rebuild the schedule on every clock tick. Parsing, validation, and compilation happen on the host.

## 18. Runtime clock

The clock tracks the active schedule and the index of the next ringing event.

When initialized, it receives the current time and skips events that have already passed.

For example, given:

```text
08:00
09:00
10:00
```

starting at `08:30` makes `09:00` the next event.

Starting after the final event leaves no remaining events for that schedule.

The clock advances through the ordered list as events occur. Its runtime logic should remain small and predictable.

## 19. Physical controls

The device provides two physical controls:

```text
[ DISABLE ]
[ MANUAL RING ]
```

**Disable** temporarily suppresses automatic ringing. It is a runtime override and does not modify the compiled schedule or source files.

**Manual Ring** triggers the bell independently of the timetable.

The electrical implementation, debounce behavior, ring duration, and interaction between the controls are hardware and runtime details.

## 20. Errors

The compiler must report invalid input explicitly.

Error categories include:

```text
UnexpectedCharacter
UnterminatedString
UnexpectedToken
InvalidTime
InvalidDate
InvalidRange
ScheduleNotFound
NoSchedule
AmbiguousSchedule
```

Errors should identify the relevant source location or value whenever practical.

Invalid dates, malformed times, reversed ranges, unresolved schedules, and ambiguous selection must not be silently corrected or ignored.

## 21. Full system

```text
                    ADMINISTRATOR
                         │
                         ▼
                    source project
                         │
              ┌──────────┴──────────┐
              │                     │
        ring.schedule           schedule/
              │                     │
       calendar rules          *.time files
              │                     │
              └──────────┬──────────┘
                         ▼
                        ringc
                         │
            ┌────────────┼────────────┐
            │            │            │
           init         check        build
            │            │            │
            ▼            ▼            ▼
         template    validation   compiled data
                                      │
                                      ▼
                                   install
                                      │
                                      ▼
                                    DEVICE
                                      │
                         ┌────────────┼────────────┐
                         │            │            │
                        RTC        schedule     controls
                         │            │        ┌───┴────┐
                         └────────────┤      disable  manual
                                      ▼
                                 ring controller
                                      │
                                      ▼
                                  bell output
```

The maintenance path is:

```text
edit source
    ↓
ringc check <path>
    ↓
ringc build <path>
    ↓
ringc update
    ↓
existing device receives new schedule
```

The central rules are:

```text
ring.schedule
    defines WHEN

*.time
    defines WHAT

firmware / framework
    provides the controller

compiled schedule
    provides the school-specific data

install
    installs infrastructure and initial data

update
    replaces schedule data without reinstalling infrastructure
```

## Compiled File

`ringc build` compiles the configuration and referenced `.time` files into a single binary file for the target device. The device must not need to parse source files, resolve dates, or access the filesystem.

### Format

The binary file contains three sections in order:

1. **Header**
   - Magic bytes identifying the file as a `ringc` schedule.
   - Format version.
   - Calendar start date.
   - Number of calendar days.
   - Number of schedules.

2. **Schedule table**
   - Each schedule has a unique `u8` ID within the compiled file.
   - Each record contains the schedule's ring count and its ringing times.
   - Ringing times are stored as `u16` minutes after midnight, from `0` to `1439`.
   - `Disable` is represented as a schedule with zero ringing times.

3. **Calendar**
   - One `u8` schedule ID per day, beginning at the header's start date.
   - Each entry identifies the schedule active on that date.
   - The calendar is fully resolved during compilation, including date ranges, weekdays, and schedule precedence.

### Constraints

- Schedule IDs are local to the compiled file and must not depend on filesystem enumeration order.
- The compiler must reject duplicate or ambiguous schedule definitions, unresolved references, invalid dates, unsupported schedule counts, and invalid ringing times.
- Every calendar entry must reference an existing schedule.
- The compiler must reject a calendar that exceeds the supported date range or cannot be represented by the format.
- The format must be versioned so incompatible changes can be detected by the device.
- The device must validate the header and section boundaries before using the file.

### Runtime

The device reads the current date from its RTC, calculates the calendar index relative to the start date, and retrieves the corresponding schedule ID. It then reads that schedule's ringing times and triggers the next applicable event.

Updating a schedule replaces the compiled file without reflashing the firmware. The firmware and schedule data remain separate.

Implementation should preserve these boundaries. Calendar parsing, schedule selection, timetable execution, firmware behavior, and physical controls have separate responsibilities.

Target device is STM32F407VE
