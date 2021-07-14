//! This module defines the stamps for dates and time.

use crate::checksum::{Digest, Hasher, Tag};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

pub type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Date(chrono::NaiveDate);

impl Date {
    pub fn year(&self) -> String {
        self.0.format("%Y").to_string()
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d").to_string())
    }
}

impl FromStr for Date {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let naive = chrono::NaiveDate::from_str(s)?;

        Ok(Self(naive))
    }
}

impl Digest for Date {
    fn digest(&self, hasher: &mut Hasher) {
        hasher.update(&Tag::Date.to_bytes());
        self.to_string().digest(hasher);
    }
}

impl Digest for DateTime {
    fn digest(&self, hasher: &mut Hasher) {
        hasher.update(&Tag::Timestamp.to_bytes());
        self.format("%Y-%m-%dT%h:%m:%sZ").to_string().digest(hasher);
    }
}

use serde::de::{self, Visitor};
struct DateVisitor;

impl<'de> Visitor<'de> for DateVisitor {
    type Value = Date;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an ISO8601 date")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        let date = Date::from_str(value).map_err(de::Error::custom)?;

        Ok(date)
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
        let date = Date::from_str(&value).map_err(de::Error::custom)?;

        Ok(date)
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        let date = Date::from_str(visitor.next_value()?).map_err(de::Error::custom)?;

        Ok(date)
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(DateVisitor)
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;

        let s = self.to_string();
        let value = toml::value::Datetime::from_str(&s).map_err(Error::custom)?;

        value.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize)]
    struct Test {
        title: String,
        date: Date,
    }

    #[test]
    fn des_toml() -> Result<()> {
        let raw = "title = \"Test\"\ndate = 2021-01-02\n";
        let test: Test = toml::from_str(raw)?;

        assert_eq!(&test.date.to_string(), "2021-01-02");

        Ok(())
    }

    #[test]
    fn ser_toml() -> Result<()> {
        let test = Test {
            title: "Test".to_string(),
            date: Date::from_str("2021-01-02")?,
        };
        let raw = "title = \"Test\"\ndate = 2021-01-02\n";
        let actual = toml::to_string(&test)?;

        assert_eq!(&actual, raw);

        Ok(())
    }

    #[test]
    fn des_json() -> Result<()> {
        let raw = r#"{"title": "Test", "date": "2021-01-02"}"#;
        let test: Test = serde_json::from_str(raw)?;

        assert_eq!(&test.date.to_string(), "2021-01-02");

        Ok(())
    }

    // #[test]
    // fn ser_json() -> Result<()> {
    //     let test = Test {
    //         title: "Test".to_string(),
    //         date: Date::from_str("2021-01-02")?,
    //     };
    //     let raw = r#"{"title": "Test", "date": "2021-01-02"}"#;
    //     let actual = serde_json::to_string(&test)?;

    //     assert_eq!(&actual, raw);

    //     Ok(())
    // }
}
