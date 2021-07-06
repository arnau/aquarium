use anyhow::Result;
use pulldown_cmark::{escape::StrWrite, Event, Parser, Tag};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum ExtractState {
    Pending,
    Active,
    Done,
}

/// A state machine to track the extraction of a piece from a Markdown text.
///
/// See [`take_title`].
#[derive(Debug, Clone, PartialEq)]
pub struct Extract<'a, I: Iterator<Item = Event<'a>>, W> {
    iter: I,
    writer: W,
    state: ExtractState,
}

impl<'a, I, W> Extract<'a, I, W>
where
    I: Iterator<Item = Event<'a>>,
    W: StrWrite,
{
    pub fn new(iter: I, writer: W) -> Self {
        Self {
            iter,
            writer,
            state: ExtractState::Pending,
        }
    }

    pub fn next(&mut self) -> Option<Event<'a>> {
        self.iter.next()
    }

    pub fn activate(&mut self) {
        if let ExtractState::Pending = self.state {
            self.state = ExtractState::Active;
        }
    }

    pub fn append(&mut self, fragment: &str) -> Result<()> {
        if let ExtractState::Active = self.state {
            self.writer.write_str(fragment)?;
        }

        Ok(())
    }

    pub fn finish(&mut self) {
        if let ExtractState::Active = self.state {
            self.state = ExtractState::Done;
        }
    }

    pub fn is_done(&self) -> bool {
        ExtractState::Done == self.state
    }

    pub fn is_active(&self) -> bool {
        ExtractState::Active == self.state
    }
}

#[derive(Debug, Clone, Error)]
pub enum ExtractError {
    #[error("Could not find a title in the given markdown text.")]
    NotFound,
}

/// Extracts the title (i.e. first h1) from a markdown text.
pub fn take_title<'a>(text: &str) -> Result<String> {
    let parser = Parser::new(text);
    let mut recipient = String::new();
    let mut extract = Extract::new(parser, &mut recipient);

    while let Some(event) = extract.next() {
        match event {
            Event::Start(Tag::Heading(1)) => {
                if extract.is_done() {
                    break;
                }
                extract.activate();
            }
            Event::End(Tag::Heading(1)) => {
                &extract.finish();
            }
            Event::Start(Tag::Emphasis) | Event::End(Tag::Emphasis) => {
                if extract.is_active() {
                    extract.append("_")?;
                }
            }
            Event::Start(Tag::Strong) | Event::End(Tag::Strong) => {
                if extract.is_active() {
                    extract.append("**")?;
                }
            }
            Event::Code(ref text) => {
                if extract.is_active() {
                    extract.append("`")?;
                    extract.append(text)?;
                    extract.append("`")?;
                }
            }
            Event::Text(ref text) => {
                if extract.is_active() {
                    extract.append(text)?;
                }
            }
            _ => (),
        }
    }

    if !extract.is_done() {
        return Err(ExtractError::NotFound.into());
    }

    Ok(recipient)
}

pub fn split_title(input: &str) -> Result<(String, String)> {
    let title = take_title(input)?;

    if let Some((_, rest)) = input.split_once(&format!("# {}", &title)) {
        return Ok((title, rest.trim().into()));
    }

    Err(ExtractError::NotFound.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_plain_title() -> Result<()> {
        let text = r#"# Nineteen Eighty-Four, George Orwell (1949)

It was a bright cold day in April, and the clocks were striking thirteen."#;
        let expected = "Nineteen Eighty-Four, George Orwell (1949)";
        let actual = take_title(text)?;

        assert_eq!(&actual, expected);
        Ok(())
    }

    #[test]
    fn extract_rich_title() -> Result<()> {
        let text = r#"# Nineteen Eighty-Four, _George Orwell_ (**1949**)

It was a bright cold day in April, and the clocks were striking thirteen."#;
        let expected = "Nineteen Eighty-Four, _George Orwell_ (**1949**)";
        let actual = take_title(text)?;

        assert_eq!(&actual, expected);
        Ok(())
    }

    #[test]
    fn no_title() {
        let text = r#"It was a bright cold day in April, and the clocks were striking thirteen."#;
        let actual = take_title(text);

        assert!(actual.is_err(), "error when no title found");
    }

    #[test]
    fn split_title_and_content() -> Result<()> {
        let text = r#"# Nineteen Eighty-Four

It was a bright cold day in April, and the clocks were striking thirteen."#;
        let expected = (
            "Nineteen Eighty-Four".into(),
            "It was a bright cold day in April, and the clocks were striking thirteen.".into(),
        );
        let actual = split_title(text)?;

        assert_eq!(actual, expected);
        Ok(())
    }
}
