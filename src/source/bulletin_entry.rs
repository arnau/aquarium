//! This module defines bulletin for the Source stage.

use serde::{Deserialize, Serialize};

use crate::cache::records::*;
use crate::checksum::{Digest, Hasher};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BulletinEntry {
    url: String,
    #[serde(deserialize_with = "super::de_trim")]
    title: String,
    #[serde(deserialize_with = "super::de_trim")]
    summary: String,
    content_type: String,
}

impl Digest for BulletinEntry {
    fn digest(&self, hasher: &mut Hasher) {
        self.url.digest(hasher);
        self.title.digest(hasher);
        self.summary.digest(hasher);
        self.content_type.digest(hasher);
    }
}

impl From<(Option<String>, &BulletinEntry)> for BulletinEntryRecord {
    fn from(pair: (Option<String>, &BulletinEntry)) -> Self {
        let (id, resource) = pair;
        Self {
            checksum: resource.checksum().to_string(),
            url: resource.url.to_string(),
            issue_id: id,
            title: resource.title.to_string(),
            summary: resource.summary.to_string(),
            content_type: resource.content_type.to_string(),
        }
    }
}

impl From<BulletinEntryRecord> for BulletinEntry {
    fn from(record: BulletinEntryRecord) -> Self {
        Self {
            url: record.url,
            title: record.title,
            summary: record.summary,
            content_type: record.content_type,
        }
    }
}
