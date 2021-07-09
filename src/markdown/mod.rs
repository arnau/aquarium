use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

mod extract;

lazy_static! {
    static ref FRONTMATTER_RE: Regex =
        Regex::new(r"^\s*---(\r?\n(?s).*?(?-s))---\r?\n?((?s).*(?-s))$").unwrap();
}

/// Represents a Markdown blob.
///
/// It expects a frontmatter in YAML, a title (i.e. H1) and a summary terminated by `<!-- body -->`.
#[derive(Debug, Clone)]
pub struct Markdown {
    pub(crate) frontmatter: String,
    pub(crate) title: String,
    pub(crate) summary: Option<String>,
    pub(crate) body: String,
}

impl FromStr for Markdown {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let blob = input.to_string();
        let (frontmatter, body) = take_frontmatter(&blob)?;
        let (title, body) = take_title(body)?;
        let (summary, body) = take_summary(&body);

        Ok(Self {
            frontmatter: frontmatter.into(),
            title,
            summary,
            body,
        })
    }
}

impl Markdown {
    pub fn frontmatter(&self) -> &str {
        &self.frontmatter
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn summary(&self) -> Option<&String> {
        self.summary.as_ref()
    }

    pub fn to_tuple(self) -> (String, String, Option<String>, String) {
        (self.frontmatter, self.title, self.summary, self.body)
    }
}

fn take_frontmatter(blob: &str) -> Result<(&str, &str)> {
    let groups = FRONTMATTER_RE
        .captures(blob)
        .expect("frontmatter split failure");
    let frontmatter = groups.get(1).expect("group frontmatter missing").as_str();
    let content = groups.get(2).expect("group content missing").as_str();

    Ok((frontmatter, content))
}

fn take_title(input: &str) -> Result<(String, String)> {
    let title = extract::take_title(input)?;

    if let Some((_, rest)) = input.split_once(&format!("# {}", &title)) {
        return Ok((title, rest.trim().into()));
    }

    Err(extract::ExtractError::NotFound.into())
}

fn take_summary(input: &str) -> (Option<String>, String) {
    if let Some((summary, body)) = input.split_once("<!-- body -->") {
        (Some(summary.trim().into()), body.trim().into())
    } else {
        (None, input.trim().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() -> Result<()> {
        let raw = r#"
---
foo: bar
---
# Title

Summary

<!-- body -->

Body.
        "#;
        let actual = Markdown::from_str(raw)?;

        assert_eq!(actual.title(), "Title");
        assert_eq!(actual.summary(), Some(&"Summary".into()));
        assert_eq!(actual.body(), "Body.");

        Ok(())
    }
}
