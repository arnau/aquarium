# Aquarium

Aquarium is a tool to manage [Seachess].


## Design

The idea is to have a staged process that allows the tranformation of the source information into each required output.

- [ ] Source: The source is a combination of Markdown, TOML and any other format that is convenient to write and maintain.
- [ ] Cache: A canonical representation of the source.
- [ ] Zola: A representation for convenient consumption by [Zola].
- [ ] Feed: A representation as RSS/Atom.
- [ ] DataPackage: A representation as [Tabular Data Package].


## Licence

The codebase is licensed under the [MIT licence](./LICENCE).


[Seachess]: https://www.seachess.net/
[Zola]: https://www.getzola.org/
[Tabular Data Package]: https://specs.frictionlessdata.io/tabular-data-package/
