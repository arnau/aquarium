//! This module defines the stamps for dates and time.

use crate::checksum::{Digest, Hasher, Tag};

pub type Date = chrono::NaiveDate;
pub type DateTime = chrono::DateTime<chrono::Utc>;

impl Digest for Date {
    fn digest(&self, hasher: &mut Hasher) {
        hasher.update(&Tag::Date.to_bytes());
        self.format("%Y-%m-%d").to_string().digest(hasher);
    }
}

impl Digest for DateTime {
    fn digest(&self, hasher: &mut Hasher) {
        hasher.update(&Tag::Timestamp.to_bytes());
        self.format("%Y-%m-%dT%h:%m:%sZ").to_string().digest(hasher);
    }
}
