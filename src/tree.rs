#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
}

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
}

#[derive(Debug, PartialEq, Eq)]
pub enum Rule {
    Default,
    Automatic,
    Always,
    Range { from: Date, to: Date },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Date {
    pub year: Option<u16>,
    pub month: Option<u8>,
    pub day: Option<u8>,
}
