use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Date {
    pub year: u16,
    pub month: Month,
    pub day: Day,
}

impl FromStr for Date {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Date, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() != "xxxx-xx-xx".len() {
            return Err("wrong number of characters");
        }

        let Some(year) = parse(&bytes[0..4]) else {
            return Err("the year part is not a number");
        };

        let bytes = &bytes[4..];
        if bytes[0] != b'-' {
            return Err("expected the '-' after the year");
        }

        let bytes = &bytes[1..];
        let Some(month) = parse(&bytes[0..2]) else {
            return Err("the month part is not a number");
        };

        let month = Month::new(month)?;

        let bytes = &bytes[2..];
        if bytes[0] != b'-' {
            return Err("expected the '-' after the month");
        }

        let bytes = &bytes[1..];
        let Some(day) = parse(&bytes[0..2]) else {
            return Err("the day part is not a number");
        };

        let day = Day::new(day)?;

        Ok(Self {
            year: year as u16,
            month,
            day,
        })
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Date) -> std::cmp::Ordering {
        fn as_tuple(d: &Date) -> (u16, u8, u8) {
            (d.year, d.month.as_u8(), d.day.as_u8())
        }

        let lhs = as_tuple(self);
        let rhs = as_tuple(other);

        lhs.cmp(&rhs)
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Date) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Month(u8);

impl Month {
    pub fn new(v: usize) -> Result<Self, &'static str> {
        if (1..=12).contains(&v) {
            Ok(Self(v as u8))
        } else {
            Err("the month must be a number in range 1..12")
        }
    }

    #[inline]
    pub const fn as_u8(&self) -> u8 {
        self.0
    }
}

impl Default for Month {
    fn default() -> Self {
        Self(1)
    }
}

impl Display for Month {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self.0 {
            1 => f.write_str("January"),
            2 => f.write_str("February"),
            3 => f.write_str("March"),
            4 => f.write_str("April"),
            5 => f.write_str("May"),
            6 => f.write_str("June"),
            7 => f.write_str("July"),
            8 => f.write_str("August"),
            9 => f.write_str("September"),
            10 => f.write_str("October"),
            11 => f.write_str("November"),
            12 => f.write_str("December"),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Day(u8);

impl Day {
    pub fn new(v: usize) -> Result<Self, &'static str> {
        if (1..=31).contains(&v) {
            Ok(Self(v as u8))
        } else {
            Err("the day must be a number in range 1..31")
        }
    }

    #[inline]
    pub const fn as_u8(&self) -> u8 {
        self.0
    }
}

impl Default for Day {
    fn default() -> Self {
        Self(1)
    }
}

fn parse(bytes: &[u8]) -> Option<usize> {
    let mut res = 0_usize;
    for b in bytes {
        let digit = match b {
            b'0' => 0,
            b'1' => 1,
            b'2' => 2,
            b'3' => 3,
            b'4' => 4,
            b'5' => 5,
            b'6' => 6,
            b'7' => 7,
            b'8' => 8,
            b'9' => 9,
            _ => return None,
        };

        res = res * 10 + digit;
    }

    Some(res)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_str() {
        let cases = [
            (
                "2025-12-23",
                Ok(Date {
                    year: 2025,
                    month: Month::new(12).unwrap(),
                    day: Day::new(23).unwrap(),
                }),
            ),
            ("202-12-23", Err("wrong number of characters")),
            ("2025?12-23", Err("expected the '-' after the year")),
            ("2025-12?23", Err("expected the '-' after the month")),
            ("2XXX-12-23", Err("the year part is not a number")),
            ("2025-XX-23", Err("the month part is not a number")),
            ("2025-12-XX", Err("the day part is not a number")),
            (
                "2025-33-01",
                Err("the month must be a number in range 1..12"),
            ),
            ("2025-02-99", Err("the day must be a number in range 1..31")),
        ];

        for (input, expected) in cases {
            let got = Date::from_str(input);
            assert_eq!(got, expected, "input = '{input}'");
        }
    }
}
