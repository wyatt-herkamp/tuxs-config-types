/*
MIT License

Copyright (c) 2023 Wyatt Herkamp

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};
use chrono::Duration;
use serde::{Deserialize, Deserializer};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use once_cell::sync::OnceCell;
use regex::Regex;
use strum::{
    AsRefStr, Display, EnumCount, EnumIs, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr,
};
use thiserror::Error;

static UNITS_REGEX: OnceCell<Regex> = OnceCell::new();
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
#[non_exhaustive]
pub enum Unit {
    #[default]
    #[strum(serialize ="ms")]
    Milliseconds,
    #[strum(serialize ="s")]
    Seconds,
    #[strum(serialize ="m")]
    Minutes,
    #[strum(serialize ="h")]
    Hours,
    #[strum(serialize ="d")]
    Days,
}
impl Unit {
    pub fn build_regex() -> Regex {
        Regex::new(&Self::create_regex_string()).expect(
            format!(
                "Unable to Build Size Config Regex. Please Report: {:?}. ",
                Self::create_regex_string()
            )
                .as_str(),
        )
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConfigDuration {
    pub duration: Duration,
    pub unit: Unit,
}
impl Display for ConfigDuration{
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
impl FromStr for ConfigDuration{
    type Err = InvalidDurationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = UNITS_REGEX.get_or_init(Unit::build_regex);
        let captures = regex
            .captures(&s)
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
impl ConfigDuration{
    pub fn into_inner(self) -> Duration{
        self.duration
    }
}

impl Into<(Duration, Unit)> for ConfigDuration {
    fn into(self) -> (Duration, Unit) {
        (self.duration, self.unit)
    }
}
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

impl Deref for ConfigDuration {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.duration
    }
}

impl DerefMut for ConfigDuration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.duration
    }
}

impl AsRef<Duration> for ConfigDuration {
    fn as_ref(&self) -> &Duration {
        &self.duration
    }
}
impl AsRef<Unit> for ConfigDuration {
    fn as_ref(&self) -> &Unit {
        &self.unit
    }
}
impl AsMut<Duration> for ConfigDuration {
    fn as_mut(&mut self) -> &mut Duration {
        &mut self.duration
    }
}

#[cfg(feature = "serde")]
pub mod impl_serde{
    use super::*;
    impl serde::Serialize for ConfigDuration {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
        {
            self.to_string().serialize(serializer)
        }
    }
    impl<'de> Deserialize<'de> for ConfigDuration {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
        {
            let value = String::deserialize(deserializer)?;
            Self::from_str(&value).map_err(serde::de::Error::custom)
        }
    }
}
