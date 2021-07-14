+++
title = "A note with a twist"
description = "This note is showing some more stuff. For the basics go to A note."
date = 2021-07-07
template = "note.html"
in_search_index = true

[extra]
id = "another-note"
title = "A note with a _twist_"
summary = "This note is showing some more stuff. For the basics go to [A note](/notes/a-note)."

[extra.author]
id = "arnau"
name = "Arnau Siches"
guest = false
+++
The summary has a curly reference to another note. Won't work with generic tools (e.g. GitHub) but should help keep
referential integrity in check. For Zola, it should become something like:

```markdown
This note is showing some more stuff. For the basics go to [A simple note](/notes/a-note.md).
```

And perhaps for Obsidian something like:

```markdown
This note is showing some more stuff. For the basics go to [[a-note]].
```


## Parsing code blocks

```dot
digraph g {
  bgcolor="#ffffff00" # RGBA (with alpha);
  rankdir = LR;

  A [ shape = egg ];
  B [ shape = egg ];

  A -> B [ label = "alternate" ];
}
```

The above should generate a SVG either inline or as an external asset.