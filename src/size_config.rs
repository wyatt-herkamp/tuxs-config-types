use derive_more::derive::{AsRef, Deref, DerefMut, From, Into};
use regex::Regex;
use std::error::Error;
use std::str::FromStr;
use std::sync::OnceLock;
use std::{cmp::Ordering, fmt::Display};
use strum::{
    AsRefStr, Display, EnumCount, EnumIs, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr,
};
use thiserror::Error;

use crate::macros::{extend_string_from_and_to, serde_via_string_types};
static UNITS_REGEX: OnceLock<Regex> = OnceLock::new();
type AnyError = Box<dyn Error + Send + Sync + 'static>;
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Display,
    EnumString,
    AsRefStr,
    EnumCount,
    IntoStaticStr,
    EnumIter,
    EnumIs,
)]
#[cfg_attr(feature = "digestible", derive(digestible::Digestible))]
#[repr(usize)]
#[non_exhaustive]
pub enum Unit {
    #[default]
    #[strum(serialize = "B")]
    Bytes = 1,
    #[strum(serialize = "KiB")]
    Kibibytes = 1024,
    #[strum(serialize = "MiB")]
    Mebibytes = 1024 * 2,
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
        let mut unit_options = String::with_capacity(Unit::iter().count() * 4); // Most Units are 3 characters long + 1 for the pipe
        let mut iter = Unit::iter().peekable();
        while let Some(value) = iter.next() {
            unit_options.push_str(value.into());
            if iter.peek().is_some() {
                unit_options.push('|');
            }
        }
        format!(r#"(?<size>[0-9]+)(?<unit>[{}]+)?"#, unit_options)
    }
}
#[derive(Debug, Error)]
#[error("{0}: {1:?}")]
pub struct InvalidSizeError(&'static str, Option<AnyError>);

impl From<(&'static str, AnyError)> for InvalidSizeError {
    fn from(value: (&'static str, AnyError)) -> Self {
        Self(value.0, Some(value.1))
    }
}
impl From<&'static str> for InvalidSizeError {
    fn from(value: &'static str) -> Self {
        Self(value, None)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, AsRef, Deref, DerefMut, Into)]
#[cfg_attr(feature = "digestible", derive(digestible::Digestible))]
pub struct ConfigSize {
    #[deref]
    #[deref_mut]
    pub size: usize,
    pub unit: Unit,
}
serde_via_string_types!(ConfigSize);
impl Display for ConfigSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.size, self.unit)
    }
}

impl FromStr for ConfigSize {
    type Err = InvalidSizeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = UNITS_REGEX.get_or_init(Unit::build_regex);
        let captures = regex
            .captures(s)
            .ok_or_else(|| InvalidSizeError::from("Does not meet requirements for a size"))?;
        let size = captures
            .name("size")
            .unwrap()
            .as_str()
            .parse::<usize>()
            .map_err(|v| InvalidSizeError::from(("Invalid Size", v.into())))?;

        let unit = captures
            .name("unit")
            .map(|unit| {
                Unit::from_str(unit.as_str())
                    .map_err(|v| InvalidSizeError::from(("Invalid Size", v.into())))
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Self { size, unit })
    }
}
extend_string_from_and_to!(ConfigSize, InvalidSizeError);
impl From<usize> for ConfigSize {
    fn from(value: usize) -> Self {
        if value % (Unit::Mebibytes as usize) == 0 {
            Self::new_from_mebibytes(value / (Unit::Mebibytes as usize))
        } else if value % (Unit::Kibibytes as usize) == 0 {
            Self::new_from_kibibytes(value / (Unit::Kibibytes as usize))
        } else {
            Self::new_from_bytes(value)
        }
    }
}
impl From<ConfigSize> for usize {
    fn from(val: ConfigSize) -> Self {
        val.size * (val.unit as usize)
    }
}

impl PartialOrd for ConfigSize {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.get_as_bytes().cmp(&other.get_as_bytes()))
    }
}
impl Ord for ConfigSize {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_as_bytes().cmp(&other.get_as_bytes())
    }
}
impl ConfigSize {
    pub fn new_from_bytes(size: usize) -> Self {
        Self {
            size,
            unit: Unit::Kibibytes,
        }
    }
    pub fn new_from_kibibytes(size: usize) -> Self {
        Self {
            size,
            unit: Unit::Kibibytes,
        }
    }
    pub fn new_from_mebibytes(size: usize) -> Self {
        Self {
            size,
            unit: Unit::Mebibytes,
        }
    }
    pub fn get_as_bytes(&self) -> usize {
        self.size * (self.unit as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;
    use strum::IntoEnumIterator;
    #[test]
    pub fn test_unit_regex() {
        println!("{}", Unit::create_regex_string());
        for unit in Unit::iter() {
            let number = rand::thread_rng().gen_range(100..10000);
            let s = format!("{}{}", number, unit);
            let from_str = ConfigSize::from_str(&s);
            assert!(from_str.is_ok());
            let captures = from_str.unwrap();
            println!("{} -> {:?}", s, captures)
        }
        let no_unit = ConfigSize::from_str("100");
        assert!(no_unit.is_ok());
        let captures = no_unit.unwrap();
        println!("100 -> {:?}", captures)
    }
    #[derive(Serialize, Deserialize)]
    pub struct SerdeTest {
        pub size: ConfigSize,
    }
    #[test]
    pub fn test_serde() {
        for unit in Unit::iter() {
            let number = rand::thread_rng().gen_range(100..10000);
            let test = SerdeTest {
                size: ConfigSize { size: number, unit },
            };
            let string = serde_json::to_string(&test).unwrap();
            let test2: SerdeTest = serde_json::from_str(&string).unwrap();
            assert_eq!(test.size, test2.size);
            println!("{:?} -> {} -> {:?}", test.size, string, test2.size)
        }
    }
}
