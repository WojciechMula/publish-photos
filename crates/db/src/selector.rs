use crate::Date;
use crate::Month;
use crate::Year;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub enum Selector {
    ByYear(Year),
    ByMonth(Year, Month),
    ByDate(Date),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub enum SelectorLevel {
    Year,
    Month,
    Day,
}

impl Selector {
    pub fn matches(&self, v: &Date) -> bool {
        match self {
            Self::ByDate(date) => date == v,
            Self::ByMonth(year, month) => *year == v.year && *month == v.month,
            Self::ByYear(year) => *year == v.year,
        }
    }

    const fn key(&self) -> (u16, u8, u8) {
        match self {
            Self::ByMonth(year, month) => (*year, month.as_u8(), u8::MAX),
            Self::ByDate(date) => (date.year, date.month.as_u8(), date.day.as_u8()),
            Self::ByYear(year) => (*year, u8::MAX, u8::MAX),
        }
    }

    pub const fn level(&self) -> SelectorLevel {
        match self {
            Self::ByYear(..) => SelectorLevel::Year,
            Self::ByMonth(..) => SelectorLevel::Month,
            Self::ByDate(..) => SelectorLevel::Day,
        }
    }

    pub fn trim_to(&self, level: SelectorLevel) -> Option<Self> {
        match self {
            Self::ByYear(..) => match level {
                SelectorLevel::Year => Some(*self),
                SelectorLevel::Month => None,
                SelectorLevel::Day => None,
            },
            Self::ByMonth(year, _) => match level {
                SelectorLevel::Year => Some(Self::ByYear(*year)),
                SelectorLevel::Month => Some(*self),
                SelectorLevel::Day => None,
            },
            Self::ByDate(date) => match level {
                SelectorLevel::Year => Some(Self::ByYear(date.year)),
                SelectorLevel::Month => Some(Self::ByMonth(date.year, date.month)),
                SelectorLevel::Day => Some(*self),
            },
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

impl SelectorLevel {
    pub const ALL: [Self; 3] = [Self::Year, Self::Month, Self::Day];

    pub const fn name(&self) -> &str {
        match self {
            Self::Year => "Year",
            Self::Month => "Month",
            Self::Day => "Day",
        }
    }

    pub const fn prev(&self) -> Option<Self> {
        match self {
            Self::Year => None,
            Self::Month => Some(Self::Year),
            Self::Day => Some(Self::Month),
        }
    }

    pub const fn next(&self) -> Option<Self> {
        match self {
            Self::Year => Some(Self::Month),
            Self::Month => Some(Self::Day),
            Self::Day => None,
        }
    }

    const fn key(&self) -> usize {
        match self {
            Self::Year => 2,
            Self::Month => 1,
            Self::Day => 0,
        }
    }
}

impl Ord for SelectorLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

impl PartialOrd for SelectorLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
