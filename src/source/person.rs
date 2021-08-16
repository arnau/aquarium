//! This module defines person for the Source stage.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;

use crate::cache::records::*;
use crate::cache::{ReadCache, Transaction, WriteCache};
use crate::checksum::{Digest, Hasher};
use crate::{Resource, ResourceSet};

/// A person resource.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Person {
    #[serde(rename = "type")]
    _type: String,
    id: String,
    name: String,
    guest: bool,
    accounts: Vec<ServiceAccount>,
}

impl Resource for Person {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Digest for Person {
    fn digest(&self, hasher: &mut Hasher) {
        self.id.digest(hasher);
        self.name.digest(hasher);
        self.accounts.digest(hasher);
    }
}

impl FromStr for Person {
    type Err = anyhow::Error;

    fn from_str(blob: &str) -> Result<Self, Self::Err> {
        let person: Person = toml::from_str(blob)?;

        Ok(person)
    }
}

impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let toml = toml::to_string(&self).expect("person to serialise as TOML");

        write!(f, "{}", &toml)
    }
}

impl From<Person> for PersonRecord {
    fn from(resource: Person) -> PersonRecord {
        PersonRecord {
            checksum: resource.checksum().to_string(),
            id: resource.id.to_string(),
            name: resource.name.to_string(),
            guest: resource.guest,
        }
    }
}

fn from_record(tx: &Transaction, record: PersonRecord) -> Result<Person> {
    let accounts: Vec<ServiceAccount> = ServiceAccountRecordSet::select(tx, record.id.clone())?
        .into_iter()
        .map(ServiceAccount::from)
        .collect();
    let resource = Person {
        _type: "person".to_string(),
        id: record.id,
        name: record.name,
        guest: record.guest,
        accounts,
    };

    Ok(resource)
}

#[derive(Clone, Debug)]
pub struct PersonSet {
    inner: Vec<Person>,
}

impl PersonSet {
    pub fn new(inner: Vec<Person>) -> Self {
        Self { inner }
    }
}

impl ReadCache for PersonSet {
    type Item = Person;

    fn find(tx: &Transaction, id: &str) -> Result<Option<Self::Item>> {
        if let Some(record) = PersonRecord::select(tx, id)? {
            let resource = from_record(tx, record)?;

            Ok(Some(resource))
        } else {
            Ok(None)
        }
    }

    fn amass(tx: &Transaction) -> Result<Self> {
        let records = PersonRecordSet::select(tx)?;
        let resources = records
            .into_iter()
            .map(|record| from_record(tx, record))
            .collect::<Result<Vec<Person>>>()?;

        Ok(Self::new(resources))
    }
}

impl WriteCache for PersonSet {
    type Item = Person;

    fn add(tx: &Transaction, resource: Self::Item) -> Result<()> {
        for account in &resource.accounts {
            let record = ServiceAccountRecord::from((resource.id.as_str(), account));
            record.insert(tx)?;
        }

        let record = PersonRecord::from(resource);
        record.insert(tx)?;

        Ok(())
    }

    fn remove(tx: &Transaction, id: &str) -> Result<()> {
        PersonRecord::delete(tx, id)
    }
}

impl ResourceSet for PersonSet {}

impl IntoIterator for PersonSet {
    type Item = Person;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl FromIterator<Person> for PersonSet {
    fn from_iter<I: IntoIterator<Item = Person>>(iter: I) -> Self {
        let mut v = Vec::new();

        for item in iter {
            v.push(item);
        }

        Self::new(v)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServiceAccount {
    id: String,
    name: String,
    username: String,
    url: String,
}

impl Digest for ServiceAccount {
    fn digest(&self, hasher: &mut Hasher) {
        self.id.digest(hasher);
        self.name.digest(hasher);
        self.username.digest(hasher);
        self.url.digest(hasher);
    }
}

impl From<(&str, &ServiceAccount)> for ServiceAccountRecord {
    fn from(pair: (&str, &ServiceAccount)) -> ServiceAccountRecord {
        let (person_id, resource) = pair;
        ServiceAccountRecord {
            checksum: resource.checksum().to_string(),
            id: resource.id.to_string(),
            person_id: person_id.to_string(),
            name: resource.name.to_string(),
            username: resource.username.to_string(),
            url: resource.url.to_string(),
        }
    }
}

impl From<ServiceAccountRecord> for ServiceAccount {
    fn from(record: ServiceAccountRecord) -> ServiceAccount {
        ServiceAccount {
            id: record.id,
            name: record.name,
            username: record.username,
            url: record.url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::Cache;

    #[test]
    fn full_cycle() -> Result<()> {
        let raw = r#"type = "person"
id = "bobsponge"
name = "Bobsponge"
guest = false

[[accounts]]
id = "github"
name = "Github"
username = "@bobsponge"
url = "https://github.com/bobsponge"

[[accounts]]
id = "mastodon"
name = "Mastodon"
username = "@bobs"
url = "https://mastodon.xyz/@bobs"
"#;
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;
        let resource = Person::from_str(raw)?;

        PersonSet::add(&tx, resource.clone())?;

        let cached = PersonSet::find(&tx, &resource.id)?.expect("resource to be cached");

        assert_eq!(&cached.to_string(), raw);
        assert_eq!(cached, resource);

        tx.commit()?;

        Ok(())
    }
}
