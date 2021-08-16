use pulldown_cmark::{Event, Options, Parser, Tag};

/// Strips all markdown from the given text.
pub fn strip(text: &str) -> String {
    let options = Options::all();

    let parser = Parser::new_ext(text, options);
    let mut recipient = String::new();

    for event in parser {
        match event {
            Event::Code(ref text) => {
                recipient.push(' ');
                recipient.push_str(text);
            }
            Event::Text(ref text) => {
                recipient.push_str(text);
            }
            Event::Html(ref text) => {
                if !text.starts_with('<') {
                    recipient.push_str(text);
                }
            }
            Event::Start(
                Tag::Emphasis | Tag::Link(..) | Tag::Strikethrough | Tag::Strong | Tag::BlockQuote,
            ) => (),
            Event::End(
                Tag::Emphasis | Tag::Link(..) | Tag::Strikethrough | Tag::Strong | Tag::BlockQuote,
            ) => (),
            _ => {
                recipient.push('\n');
            }
        }
    }

    recipient.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_inline() {
        let text = r#"A thing _with_ some **markdown** in it."#;
        let expected = "A thing with some markdown in it.";
        let actual = strip(text);

        assert_eq!(&actual, expected);
    }

    #[test]
    fn strip_html() {
        let text = r#"
# A title

A summary with _some_ ~~flavour~~.

---

## A subtitle

```
fn stuff() {}
```

<div>
text inside html
</div>
"#;
        let expected = r#"A title

A summary with some flavour.


A subtitle

fn stuff() {}

text inside html"#;
        let actual = strip(text);

        assert_eq!(&actual, expected);
    }

    #[test]
    fn strip_mark() {
        let text = r#"
# A title

A summary with _some_ ~~flavour~~.

<!-- body -->

Rest
"#;
        let expected = r#"A title

A summary with some flavour.

Rest"#;
        let actual = strip(text);

        assert_eq!(&actual, expected);
    }

    #[test]
    fn strip_deep_html() {
        let text = r#"A summary with <div>html <em>deep <strong>deep</strong> one</em></div>"#;
        let expected = r#"A summary with html deep deep one"#;
        let actual = strip(text);

        assert_eq!(&actual, expected);
    }

    #[test]
    fn strip_html_with_markdown() {
        let text = r#"A summary with <div>html _deep **deep** one_</div>"#;
        let expected = r#"A summary with html deep deep one"#;
        let actual = strip(text);

        assert_eq!(&actual, expected);
    }
}
