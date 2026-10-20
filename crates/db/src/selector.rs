use crate::Date;
use crate::Month;
use crate::Year;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub enum Selector {
    ByDate(Date),
    ByMonth(Year, Month),
    ByYear(Year),
}

impl Selector {
    pub fn matches(&self, v: &Date) -> bool {
        match self {
            Self::ByDate(date) => date == v,
            Self::ByMonth(year, month) => *year == v.year && *month == v.month,
            Self::ByYear(year) => *year == v.year,
        }
    }

    fn key(&self) -> (u16, u8, u8) {
        match self {
            Self::ByMonth(year, month) => (*year, month.as_u8(), u8::MAX),
            Self::ByDate(date) => (date.year, date.month.as_u8(), date.day.as_u8()),
            Self::ByYear(year) => (*year, u8::MAX, u8::MAX),
        }
    }

    pub fn as_iso_date(&self) -> String {
        match self {
            Self::ByMonth(year, month) => format!("{}-{:02}", year, month.as_u8()),
            Self::ByDate(date) => format!(
                "{}-{:02}-{:02}",
                date.year,
                date.month.as_u8(),
                date.day.as_u8()
            ),
            Self::ByYear(year) => year.to_string(),
        }
    }
}

impl Ord for Selector {
    fn cmp(&self, other: &Self) -> Ordering {
        other.key().cmp(&self.key())
    }
}

impl PartialOrd for Selector {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
