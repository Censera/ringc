//! Compiled file header (plan section 22). All integers are little-endian.

pub const MAGIC: [u8; 4] = *b"RNGC";
pub const VERSION: u8 = 1;
pub const HEADER_LEN: usize = 12;

/// Schedule ID 0 is the built-in empty schedule.
pub const EMPTY_SCHEDULE_ID: u8 = 0;

/// Times are minutes after midnight, below this value.
pub const MINUTES_PER_DAY: u16 = 1440;

#[derive(Debug, PartialEq, Eq)]
pub enum HeaderError {
    TooShort,
    BadMagic,
    UnsupportedVersion(u8),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub start_year: u16,
    pub start_month: u8,
    pub start_day: u8,
    pub day_count: u16,
    pub schedule_count: u8,
}

impl Header {
    pub fn parse(bytes: &[u8]) -> Result<Header, HeaderError> {
        if bytes.len() < HEADER_LEN {
            return Err(HeaderError::TooShort);
        }
        if bytes[0..4] != MAGIC {
            return Err(HeaderError::BadMagic);
        }
        if bytes[4] != VERSION {
            return Err(HeaderError::UnsupportedVersion(bytes[4]));
        }

        Ok(Header {
            start_year: u16::from_le_bytes([bytes[5], bytes[6]]),
            start_month: bytes[7],
            start_day: bytes[8],
            day_count: u16::from_le_bytes([bytes[9], bytes[10]]),
            schedule_count: bytes[11],
        })
    }

    pub fn write(&self, out: &mut [u8; HEADER_LEN]) {
        out[0..4].copy_from_slice(&MAGIC);
        out[4] = VERSION;
        out[5..7].copy_from_slice(&self.start_year.to_le_bytes());
        out[7] = self.start_month;
        out[8] = self.start_day;
        out[9..11].copy_from_slice(&self.day_count.to_le_bytes());
        out[11] = self.schedule_count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let header = Header {
            start_year: 2026,
            start_month: 9,
            start_day: 7,
            day_count: 293,
            schedule_count: 4,
        };
        let mut bytes = [0u8; HEADER_LEN];
        header.write(&mut bytes);
        assert_eq!(Header::parse(&bytes), Ok(header));
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(Header::parse(&[0; 4]), Err(HeaderError::TooShort));

        let mut bytes = [0u8; HEADER_LEN];
        assert_eq!(Header::parse(&bytes), Err(HeaderError::BadMagic));

        bytes[0..4].copy_from_slice(&MAGIC);
        bytes[4] = 9;
        assert_eq!(
            Header::parse(&bytes),
            Err(HeaderError::UnsupportedVersion(9))
        );
    }
}
