use anyhow::Result;
use pulldown_cmark::{Alignment, CodeBlockKind, Event, Options, Parser, Tag};
use std::io::Write;
use std::process::{Command, Stdio};

/// Processes the given markdown text with tranformation rules such as generating dot diagrams from code blocks.
pub fn enrich<'a>(text: &str) -> Result<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    // TODO: Make stack contain a dedicated enum that can hold the information as needed.
    let mut stack: Vec<Tag<'_>> = Vec::new();
    // TODO: slurp into stack
    let mut list_depth: Option<usize> = None;

    let mut parser = Parser::new_ext(text, options);
    let mut recipient = String::new();

    while let Some(event) = parser.next() {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                recipient.push_str("\n");

                match info.as_ref() {
                    "dot" => {
                        let opening = format!("\n<div class=\"figure from-{}\">", info);
                        recipient.push_str(&opening);
                    }
                    "csv target=table" => {
                        let tokens: Vec<_> = info.split(' ').collect();
                        let lang = tokens[0];
                        let opening = format!("\n<div class=\"table-wrapper from-{}\">", lang);
                        recipient.push_str(&opening);
                    }
                    "csv target=card" => {
                        let tokens: Vec<_> = info.split(' ').collect();
                        let lang = tokens[0];
                        let opening = format!("\n<div class=\"card-wrapper from-{}\">", lang);
                        recipient.push_str(&opening);
                    }
                    // TODO:
                    // "toml target=card" => {
                    //     let tokens: Vec<_> = info.split(' ').collect();
                    //     let lang = tokens[0];
                    //     let opening = format!("\n<div class=\"card-wrapper from-{}\">", lang);
                    //     recipient.push_str(&opening);
                    // }
                    _ => {
                        recipient.push_str("```");
                        recipient.push_str(&info);
                    }
                }
                recipient.push_str("\n");
                stack.push(Tag::CodeBlock(CodeBlockKind::Fenced(info)));
            }
            Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                match info.as_ref() {
                    "dot" => {
                        recipient.push_str("</div>\n");
                    }
                    "csv target=table" => {
                        recipient.push_str("</div>\n");
                    }
                    _ => {
                        recipient.push_str("```\n");
                    }
                }
                stack.pop();
            }
            Event::Start(tag @ Tag::CodeBlock(CodeBlockKind::Indented)) => {
                recipient.push_str("\n");
                recipient.push_str("```");
                recipient.push_str("\n");
                stack.push(tag);
            }
            Event::End(Tag::CodeBlock(CodeBlockKind::Indented)) => {
                recipient.push_str("```\n");
                stack.pop();
            }
            Event::Text(ref text) => {
                match stack.last() {
                    Some(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => match info.as_ref() {
                        "dot" => {
                            let svg = process_graphviz(&text)?;
                            recipient.push_str(&svg);
                        }
                        "csv target=table" => {
                            recipient.push_str("!!!!\n");
                            recipient.push_str(&text);
                            recipient.push_str("\n!!!!");
                        }
                        _ => {
                            recipient.push_str(text);
                        }
                    },
                    _ => {
                        recompose_sentence(&text, &mut recipient);
                    }
                };
            }

            Event::Code(text) => {
                // TODO: Debug why an emphasis followed by inline code yields two spaces in between.
                if recipient.ends_with("  ") {
                    recipient.pop();
                }
                recipient.push_str("`");
                recipient.push_str(&text);
                recipient.push_str("`");
                recipient.push_str(" ");
            }

            Event::Start(tag) => {
                match tag {
                    Tag::Paragraph => match stack.last() {
                        // A blockquote has one or more paragraphs and each paragraph requires a symbol.
                        //
                        // Check both closing paragraph and closing blockquote for the full picture.
                        Some(Tag::BlockQuote) => {
                            recipient.push_str("\n> ");
                        }
                        Some(Tag::FootnoteDefinition(_)) => {
                            stack.push(tag);
                        }
                        _ => {
                            recipient.push_str("\n");
                            stack.push(tag);
                        }
                    },
                    Tag::Heading(level) => {
                        let symbol = format!("\n{} ", "#".repeat(level as usize));
                        recipient.push_str(&symbol);
                        stack.push(tag);
                    }
                    Tag::BlockQuote => {
                        stack.push(tag);
                    }
                    Tag::List(_kind) => {
                        stack.push(tag);
                        list_depth = if let Some(x) = list_depth {
                            Some(x + 1)
                        } else {
                            Some(0)
                        }
                    }
                    Tag::Item => {
                        let hint = stack.pop();
                        let indent = 2;
                        let mut depth = " ".repeat(list_depth.map(|x| x * indent).unwrap_or(0));

                        // Sublists of ordered lists need an extra indenting space to compensate for the `.`.
                        if let Some(Tag::List(Some(_))) = stack.last() {
                            depth.push_str(" ");
                        }

                        if recipient.ends_with(" ") {
                            recipient.pop();
                        }

                        match hint {
                            Some(Tag::List(Some(n))) => {
                                let symbol = format!("\n{}{}. ", depth, n);
                                recipient.push_str(&symbol);
                                stack.push(Tag::List(Some(n + 1)));
                            }
                            Some(Tag::List(None)) => {
                                let symbol = format!("\n{}- ", depth);
                                recipient.push_str(&symbol);
                                stack.push(hint.unwrap());
                            }
                            _ => unreachable!(),
                        }
                    }
                    Tag::FootnoteDefinition(ref label) => {
                        recipient.push_str("\n[^");
                        recipient.push_str(label);
                        recipient.push_str("]: ");
                        stack.push(tag);
                    }
                    Tag::Table(_) => {
                        recipient.push_str("\n");
                        stack.push(tag);
                    }
                    Tag::TableHead => {
                        recipient.push_str("| ");
                    }
                    Tag::TableRow => {
                        recipient.push_str("| ");
                    }
                    Tag::TableCell => {}
                    Tag::Emphasis => {
                        stack.push(tag);
                        recipient.push_str("_");
                    }
                    Tag::Strong => {
                        stack.push(tag);
                        recipient.push_str("**");
                    }
                    Tag::Strikethrough => {
                        stack.push(tag);
                        recipient.push_str("~~");
                    }
                    Tag::Link(_kind, ref _url, ref _title) => {
                        stack.push(tag);
                        recipient.push_str("[");
                    }
                    Tag::Image(_, url, _title) => {
                        recipient.push_str("![");
                        recipient.push_str("](");
                        recipient.push_str(&url);
                        recipient.push_str(")");
                    }
                    _t => (),
                };
            }

            Event::End(tag) => {
                match tag {
                    Tag::Paragraph => {
                        if recipient.ends_with(" ") {
                            recipient.pop();
                        }

                        match stack.last() {
                            Some(Tag::BlockQuote) => {
                                recipient.push_str("\n>");
                            }
                            _ => {
                                recipient.push_str("\n");
                                stack.pop();
                            }
                        };
                    }
                    Tag::Heading(_) => {
                        recipient.pop();
                        recipient.push_str("\n");
                        stack.pop();
                    }
                    Tag::CodeBlock(_kind) => {
                        recipient.push_str("```\n");
                    }
                    Tag::FootnoteDefinition(_) => {
                        stack.pop();
                    }
                    Tag::Table(_) => {
                        stack.pop();
                        recipient.push_str("\n");
                    }
                    Tag::TableHead => match stack.last() {
                        Some(Tag::Table(alignments)) => {
                            let mark_set: Vec<&str> = alignments
                                .iter()
                                .map(|alignment| match alignment {
                                    Alignment::None => "-",
                                    Alignment::Left => ":-",
                                    Alignment::Center => ":-:",
                                    Alignment::Right => "-:",
                                })
                                .collect();
                            recipient.pop();
                            let mark = format!("\n|{}|\n", mark_set.join("|"));
                            recipient.push_str(&mark);
                        }
                        _ => unreachable!(),
                    },
                    Tag::TableRow => {
                        recipient.pop();
                        recipient.push_str("\n");
                    }
                    Tag::TableCell => {
                        recipient.push_str("| ");
                    }
                    Tag::BlockQuote => {
                        // Removes the final '>' added by the last paragraph in the blockquote.
                        recipient.pop();
                        recipient.push_str("\n");
                        stack.pop();
                    }
                    Tag::List(_) => {
                        stack.pop();
                        list_depth = if let Some(x) = list_depth {
                            if x == 0 {
                                None
                            } else {
                                Some(x - 1)
                            }
                        } else {
                            None
                        };

                        if list_depth.is_none() {
                            // Trims hanging space.
                            recipient.pop();
                            recipient.push_str("\n");
                        }
                    }
                    Tag::Item => {}
                    Tag::Emphasis => {
                        recipient.pop();
                        recipient.push_str("_ ");
                        stack.pop();
                    }
                    Tag::Strong => {
                        recipient.pop();
                        recipient.push_str("** ");
                        stack.pop();
                    }
                    Tag::Strikethrough => {
                        recipient.pop();
                        recipient.push_str("~~ ");
                        stack.pop();
                    }
                    Tag::Link(_, url, _) => {
                        recipient.pop();
                        recipient.push_str("](");
                        recipient.push_str(&url);
                        recipient.push_str(") ");
                        stack.pop();
                    }
                    Tag::Image(_, _, _) => {
                        recipient.push_str(" ");
                    }
                };
            }

            Event::Html(node) => {
                recipient.push_str(&node);
            }

            _ => (),
        }
    }

    Ok(recipient)
}

fn process_graphviz(input: &str) -> Result<String> {
    let mut child = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        let result = String::from_utf8(output.stdout)?;

        return Ok(result);
    }

    anyhow::bail!(String::from_utf8(output.stderr)?);
}

/// Recomposing paragraph chunks ensuring spaces are correct.
fn recompose_sentence(chunk: &str, recipient: &mut String) {
    let text = chunk.trim();

    // dbg!(&recipient);
    // dbg!(&text);

    if text.starts_with(".")
        || text.starts_with(",")
        || text.starts_with(";")
        || text.starts_with("]")
        || text.starts_with(")")
        || text.starts_with("}")
        || text.starts_with("“")
        || text.starts_with("”")
        || text.starts_with("‘")
        || text.starts_with("”")
    {
        if recipient.ends_with(" ") {
            recipient.pop();
        }
    }

    recipient.push_str(text);

    if !(text.ends_with("[")
        || text.ends_with("(")
        || text.ends_with("{")
        || text.ends_with("“")
        || text.ends_with("‘"))
    {
        recipient.push_str(" ");
    }
    // dbg!(&recipient);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_digraph() -> Result<()> {
        let text = "
digraph g {
  A -> B;
}";
        let actual = process_graphviz(text)?;

        assert!(actual.len() > 0);

        Ok(())
    }

    #[test]
    fn preserve_heading() -> Result<()> {
        let text = r#"# Heading

A paragraph with **inline** `stuff`.

## Heading 2

A final paragraph"#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_rich_heading() -> Result<()> {
        let text = r#"# Heading `stuff`

A paragraph with `stuff`."#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_untyped_codeblock() -> Result<()> {
        let text = r#"```
digraph g {
  A -> B;
}
```"#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_typed_unknown_codeblock() -> Result<()> {
        let text = r#"```notme
digraph g {
  A -> B;
}
```"#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_surrounded_codeblock() -> Result<()> {
        let text = r#"A paragraph.

```notme
digraph g {
  A -> B;
}

Another paragraph.
```"#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_blockquote_paragraphs() -> Result<()> {
        let text = r#"> This is a blockquote
> with more than one line.
> and _inline_ marks, and [a link](foo.html)
>
> Final blockquote paragraph."#;
        let expected = r#"> This is a blockquote with more than one line. and _inline_ marks, and [a link](foo.html)
>
> Final blockquote paragraph."#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), expected);

        Ok(())
    }

    #[test]
    fn parse_graphviz() -> Result<()> {
        let text = r#"# A dot

```dot
digraph g {
  A -> B;
}
```
"#;
        let actual = enrich(text);

        assert!(actual.is_ok());

        Ok(())
    }

    #[test]
    fn preserve_plain_links() -> Result<()> {
        let text = r#"[text](http://foo.bar)"#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_links_with_inline() -> Result<()> {
        let text = r#"[text _and_ more text and `code`](http://foo.bar)"#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_referenced_links() -> Result<()> {
        let text = r#"[text]

[text]: http://foo.bar"#;
        let expected = "[text](http://foo.bar)";
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), expected);

        Ok(())
    }

    #[test]
    fn preserve_referenced_links_with_square_parens() -> Result<()> {
        let text = r#"[[text]]

[text]: http://foo.bar"#;
        let expected = "[[text](http://foo.bar)]";
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), expected);

        Ok(())
    }

    #[test]
    fn preserve_links_with_square_parens() -> Result<()> {
        let text = r#"[[text](http://foo.bar)]"#;
        let expected = "[[text](http://foo.bar)]";
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), expected);

        Ok(())
    }

    #[test]
    fn preserve_links_in_sentences() -> Result<()> {
        let text = r#"A [link](http://foo.bar). And a text."#;

        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_multiline_paragraphs() -> Result<()> {
        let text = r#"A bit of text
then more with [a link](http://foo.bar)
and yet another line"#;
        let expected =
            r#"A bit of text then more with [a link](http://foo.bar) and yet another line"#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), expected);

        Ok(())
    }

    #[test]
    fn preserve_rich_paragraphs() -> Result<()> {
        let text = r#"A paragraph with ~~strikethrough~~, _emphasis_ and **strong**. As well as `code` and a [link](https://foo)."#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_paragraph_with_marks() -> Result<()> {
        let text = r#"In words of the RFC8288, “[...] a link is a typed connection between two resources [...]”."#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_images() -> Result<()> {
        let text = r#"A paragraph

![](foo.png)"#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_unordered_list() -> Result<()> {
        let text = r#"- item1
- item2
- item3"#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_unordered_sublist() -> Result<()> {
        let text = r#"- item1
- item2
  - item21
  - item22
- item3"#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_rich_ordered_sublist() -> Result<()> {
        let text = r#"1. item1
2. item2
   - item21 _with_ some **rich** content.
   - item22
3. item3"#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_list_with_paragraph() -> Result<()> {
        let text = r#"- item1
- item2
- item3

A paragraph."#;
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), text);

        Ok(())
    }

    #[test]
    fn preserve_basic_table() -> Result<()> {
        let text = r#"A basic table:

| foo | bar |
| --- | --- |
| baz | bim |"#;
        let expected = "A basic table:\n\n| foo | bar |\n|-|-|\n| baz | bim |";
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), expected);

        Ok(())
    }

    #[test]
    fn preserve_table_alignments() -> Result<()> {
        let text = r#"A basic table:

| foo | bar |
| :-- | --: |
| baz | bim |"#;
        let expected = "A basic table:\n\n| foo | bar |\n|:-|-:|\n| baz | bim |";
        let actual = enrich(text)?;

        assert_eq!((&actual).trim(), expected);

        Ok(())
    }

    // TODO: Implement
    #[test]
    fn process_csv_table() -> Result<()> {
        let text = r#"```csv target=table
foo,bar
baz,bim
```"#;
        let expected = r#"<div class="table-wrapper from-csv">
<table>
<thead>
   <tr>
     <th>foo</th>
     <th>bar</th>
   </tr>
</thead>
<tbody>
   <tr>
     <td>baz</td>
     <td>bim</td>
   </tr>
</tbody>
</table>
</div>"#;

        // let actual = enrich(text)?;

        // assert_eq!((&actual).trim(), expected);

        Ok(())
    }
}
