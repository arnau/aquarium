use blake3::{self, Hash};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Checksum(String);

impl Checksum {
    pub fn new(bytes: &[u8]) -> Self {
        let checksum = blake3::hash(bytes).to_hex().to_string();

        Self(checksum)
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl From<&str> for Checksum {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<&String> for Checksum {
    fn from(s: &String) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Checksum {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<Hash> for Checksum {
    fn from(h: Hash) -> Checksum {
        Checksum(h.to_hex().to_string())
    }
}

pub struct Hasher {
    inner: blake3::Hasher,
}

impl Hasher {
    pub fn new() -> Hasher {
        Hasher {
            inner: blake3::Hasher::new(),
        }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.inner.update(bytes);
    }

    pub fn finalize(&mut self) -> Checksum {
        self.inner.finalize().into()
    }
}

/// Interface to implement tagged hashing a-la objecthash.
pub trait Digest {
    fn digest(&self, hasher: &mut Hasher);
}

impl Digest for String {
    fn digest(&self, hasher: &mut Hasher) {
        hasher.update(&Tag::Unicode.to_bytes());
        hasher.update(self.as_bytes());
    }
}

impl Digest for str {
    fn digest(&self, hasher: &mut Hasher) {
        hasher.update(&Tag::Unicode.to_bytes());
        hasher.update(self.as_bytes());
    }
}

impl Digest for u32 {
    fn digest(&self, hasher: &mut Hasher) {
        hasher.update(&Tag::Integer.to_bytes());
        hasher.update(self.to_string().as_bytes());
    }
}

impl Digest for [u8] {
    fn digest(&self, hasher: &mut Hasher) {
        hasher.update(&Tag::Raw.to_bytes());
        hasher.update(self);
    }
}

impl<T: Digest> Digest for Vec<T> {
    fn digest(&self, hasher: &mut Hasher) {
        hasher.update(&Tag::List.to_bytes());
        for item in self {
            item.digest(hasher);
        }
    }
}

impl<T: Digest> Digest for Option<T> {
    fn digest(&self, hasher: &mut Hasher) {
        match self {
            None => {
                hasher.update(&Tag::Null.to_bytes());
                hasher.update(b"");
            }

            Some(v) => {
                &v.digest(hasher);
            }
        }
    }
}

impl Digest for DateTime<Utc> {
    fn digest(&self, hasher: &mut Hasher) {
        hasher.update(&Tag::Timestamp.to_bytes());
        self.format("%Y-%m-%d").to_string().digest(hasher);
    }
}

/// Tags are the same found in Objecthash except for [`Tag::Timestamp`].
#[derive(Debug, Clone, Copy)]
pub enum Tag {
    Bool = 0x62,
    Dict = 0x64,
    Float = 0x66,
    Integer = 0x69,
    List = 0x6C,
    Null = 0x6E,
    Raw = 0x72,
    Timestamp = 0x74,
    Unicode = 0x75,
}

impl Tag {
    pub fn to_bytes(&self) -> [u8; 1] {
        [*self as u8]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_byte() {
        assert_eq!(Tag::Unicode.to_bytes(), [0x75; 1])
    }
}
