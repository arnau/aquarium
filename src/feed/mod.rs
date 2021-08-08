//! This module covers both the general feed.

use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, Utc};
use rss::{Channel, ChannelBuilder, Guid, ItemBuilder};
use std::fs::File;
use std::path::Path;
use std::str::FromStr;

use crate::cache::{params, Cache, Transaction};
use crate::markdown;
// TODO: Decouple
use crate::zola::settings;

pub fn write(sink_dir: &Path, cache: &mut Cache) -> Result<()> {
    let file = File::create(sink_dir.join("rss.xml"))?;
    let tx = cache.transaction()?;
    let channel = build(&tx)?;

    channel.write_to(file)?;
    tx.commit()?;

    Ok(())
}

fn build(tx: &Transaction) -> Result<Channel> {
    let settings = settings::find(tx, "main")?.expect("Missing main settings.");
    let mut items = Vec::new();
    let query = r#"
        SELECT
            *
        FROM
            feed
        ORDER BY
            date DESC
        LIMIT 10
    "#;
    let mut stmt = tx.prepare(query)?;
    let mut rows = stmt.query(params![])?;

    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let summary: Option<String> = row.get(2)?;
        let section: String = row.get(3)?;
        let date: String = row.get(4)?;
        let pub_date = DateTime::<Utc>::from_utc(NaiveDate::from_str(&date)?.and_hms(0, 0, 0), Utc);
        let url = format!("{}/{}/{}", &settings.url, &section, &id);
        let mut guid = Guid::default();
        guid.set_value(&url);
        guid.set_permalink(true);

        let item = ItemBuilder::default()
            .title(title)
            .description(summary.map(|s| markdown::to_html(&s)))
            .link(url.clone())
            .guid(guid)
            .pub_date(pub_date.to_rfc2822())
            .build()
            .map_err(|err| anyhow!(err))?;

        items.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(&settings.title)
        .link(&settings.url)
        .description(&settings.description)
        .copyright(settings.copyright)
        .language("en".to_string())
        .generator("Aquarium".to_string())
        .items(items)
        .build()
        .map_err(|err| anyhow!(err))?;

    Ok(channel)
}
