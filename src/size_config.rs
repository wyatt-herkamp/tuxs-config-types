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
use once_cell::sync::OnceCell;
use regex::Regex;
use std::cmp::Ordering;
use std::error::Error;
use std::str::FromStr;
use strum::{
    AsRefStr, Display, EnumCount, EnumIs, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr,
};
use thiserror::Error;
static UNITS_REGEX: OnceCell<Regex> = OnceCell::new();
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConfigSize {
    pub size: usize,
    pub unit: Unit,
}
impl AsRef<Unit> for ConfigSize {
    fn as_ref(&self) -> &Unit {
        &self.unit
    }
}

impl FromStr for ConfigSize {
    type Err = InvalidSizeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = UNITS_REGEX.get_or_init(Unit::build_regex);
        let captures = regex
            .captures(&s)
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
impl Into<usize> for ConfigSize {
    fn into(self) -> usize {
        self.size * (self.unit as usize)
    }
}
impl Into<u64> for ConfigSize {
    fn into(self) -> u64 {
        self.size as u64 * (self.unit as u64)
    }
}
impl Into<u32> for ConfigSize {
    fn into(self) -> u32 {
        self.size as u32 * (self.unit as u32)
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
#[doc(hidden)]
#[cfg(feature = "serde")]
pub mod serde_impl {
    use super::ConfigSize;
    use serde::de::Error;
    use serde::{Deserialize, Deserializer};
    use std::str::FromStr;
    pub fn serialize_as_u64<S>(size: &ConfigSize, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
    {
        serializer.serialize_u64(size.get_as_bytes() as u64)
    }
    impl serde::Serialize for ConfigSize {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            format!("{}{}", self.size, self.unit).serialize(serializer)
        }
    }
    struct Visitor;
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = ConfigSize;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string representing a size")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            ConfigSize::from_str(value).map_err(Error::custom)
        }
        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            ConfigSize::from_str(v).map_err(Error::custom)
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: Error,
        {
            ConfigSize::from_str(&v).map_err(Error::custom)
        }
    }
    impl<'de> Deserialize<'de> for ConfigSize {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(Visitor)
        }
    }
}
#[cfg(test)]
mod tests {
    use rand::Rng;
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;
    use strum::IntoEnumIterator;
    use super::*;
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
