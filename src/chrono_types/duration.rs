use chrono::Duration;
use derive_more::derive::{AsRef, Deref, DerefMut, From, Into};
use regex::Regex;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::OnceLock;
use strum::{
    AsRefStr, Display, EnumCount, EnumIs, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr,
};
use thiserror::Error;

use crate::macros::{extend_string_from_and_to, serde_via_string_types};

static UNITS_REGEX: OnceLock<Regex> = OnceLock::new();
type AnyError = Box<dyn Error + Send + Sync + 'static>;
#[derive(Debug, Error)]
#[error("{0}: {1:?}")]
pub struct InvalidDurationError(&'static str, Option<AnyError>);
impl From<(&'static str, AnyError)> for InvalidDurationError {
    fn from(value: (&'static str, AnyError)) -> Self {
        Self(value.0, Some(value.1))
    }
}
impl From<&'static str> for InvalidDurationError {
    fn from(value: &'static str) -> Self {
        Self(value, None)
    }
}
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Hash,
    Display,
    EnumString,
    AsRefStr,
    EnumCount,
    IntoStaticStr,
    EnumIter,
    EnumIs,
)]
#[repr(usize)]
#[cfg_attr(feature = "digestible", derive(digestible::Digestible))]
#[non_exhaustive]
pub enum Unit {
    #[default]
    #[strum(serialize = "ms")]
    Milliseconds,
    #[strum(serialize = "s")]
    Seconds,
    #[strum(serialize = "m")]
    Minutes,
    #[strum(serialize = "h")]
    Hours,
    #[strum(serialize = "d")]
    Days,
}
serde_via_string_types!(Unit);

impl Unit {
    pub fn build_regex() -> Regex {
        Regex::new(&Self::create_regex_string())
            .map_err(|err| {
                format!(
                    "Unable to Build Size Config Regex. Please Report: {:?}. \n{:?}",
                    Self::create_regex_string(),
                    err
                )
            })
            .unwrap()
    }

    fn create_regex_string() -> String {
        let mut unit_options = String::with_capacity(Unit::iter().count() * 2); // Most Units are 1 characters long + 1 for the pipe
        let mut iter = Unit::iter().peekable();
        while let Some(value) = iter.next() {
            unit_options.push_str(value.into());
            if iter.peek().is_some() {
                unit_options.push('|');
            }
        }
        format!(r#"(?<length>[0-9]+)(?<unit>[{}]+)?"#, unit_options)
    }
}

/// A wrapper around `chrono::Duration` that allows for deserializing from a string
///
/// | Duration Type | Suffix | Example |
/// |---------------|--------|---------|
/// | milliseconds  | ms     | "100ms" |
/// | Seconds       | s      | '100s"  |
/// | Minutes       | m      | "100m"  |
/// | Hours         | h      | "100h"  |
/// | Days          | d      | "10d"   |
///
/// # Examples in TOML
/// ```toml
/// duration_in_milliseconds = "100ms"
/// duration_in_seconds = "100s"
/// duration_in_minutes = "100m"
/// duration_in_hours = "100h"
/// duration_in_days = "10d"
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, AsRef, Deref, DerefMut)]
#[cfg_attr(feature = "digestible", derive(digestible::Digestible))]
pub struct ConfigDuration {
    #[cfg_attr(feature = "digestible", digestible(digest_with = digest_with_hash))]
    #[deref]
    #[deref_mut]
    pub duration: Duration,
    pub unit: Unit,
}
serde_via_string_types!(ConfigDuration);
impl Display for ConfigDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let length = match self.unit {
            Unit::Milliseconds => self.duration.num_milliseconds(),
            Unit::Seconds => self.duration.num_seconds(),
            Unit::Minutes => self.duration.num_minutes(),
            Unit::Hours => self.duration.num_hours(),
            Unit::Days => self.duration.num_days(),
        };

        write!(f, "{}{}", length, self.unit)
    }
}
impl FromStr for ConfigDuration {
    type Err = InvalidDurationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO:  Support for more complex durations like "1h30m"
        let regex = UNITS_REGEX.get_or_init(Unit::build_regex);
        let captures = regex
            .captures(s)
            .ok_or_else(|| InvalidDurationError::from("Unable to parse duration"))?;
        let length = captures
            .name("length")
            .unwrap()
            .as_str()
            .parse::<usize>()
            .map_err(|v| InvalidDurationError::from(("Invalid Size", v.into())))?;

        let unit = captures
            .name("unit")
            .map(|unit| {
                Unit::from_str(unit.as_str())
                    .map_err(|v| InvalidDurationError::from(("Invalid Size", v.into())))
            })
            .transpose()?
            .unwrap_or_default();

        let duration = match unit {
            Unit::Milliseconds => Duration::milliseconds(length as i64),
            Unit::Seconds => Duration::seconds(length as i64),
            Unit::Minutes => Duration::minutes(length as i64),
            Unit::Hours => Duration::hours(length as i64),
            Unit::Days => Duration::days(length as i64),
        };
        Ok(Self { duration, unit })
    }
}
extend_string_from_and_to!(ConfigDuration, InvalidDurationError);
impl ConfigDuration {
    pub fn into_inner(self) -> Duration {
        self.duration
    }
}
#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for ConfigDuration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.duration.partial_cmp(&other.duration)
    }
}
impl Ord for ConfigDuration {
    fn cmp(&self, other: &Self) -> Ordering {
        self.duration.cmp(&other.duration)
    }
}
impl From<Duration> for ConfigDuration {
    fn from(duration: Duration) -> Self {
        // TODO: Make this the biggest unit possible
        Self {
            duration,
            unit: Unit::Milliseconds,
        }
    }
}
impl From<ConfigDuration> for Duration {
    fn from(duration: ConfigDuration) -> Self {
        duration.duration
    }
}
