+++
title = "Dive into CSV on the Web"
description = "This note captures my personal views on “CSV on the Web”, a specification to\nprovide metadata for CSV files."
date = 2020-10-07
template = "note.html"
in_search_index = true

[extra]
id = "dive-into-csvw"
title = "Dive into CSV on the Web"
summary = "This note captures my personal views on “CSV on the Web”, a specification to\nprovide metadata for CSV files."

[extra.author]
id = "arnau"
name = "Arnau Siches"
guest = false
+++

The _CSV on the Web_ (CSVW) is composed by four long specifications covering a wide range of concerns including but not restricted to structural information, discoverability, parsing hints, tranformation hints and contextual annotations.

In this note I'll be assessing CSVW under the assumption that it is a reasonable choice for increasing the reliability of CSV data consumption. If you are in a rush, go straight to the [closing thoughts](#closing-thoughts).

## Overview

What does it mean to publish “CSV on the Web”?

- Publish your CSV file such that it can be retrieved with a `text/csv` content type.
- Publish a metadata JSON file that conforms to the [metadata vocabulary](https://www.w3.org/TR/2015/REC-tabular-metadata-20151217/) and [tabular data model](https://www.w3.org/TR/2015/REC-tabular-data-model-20151217/).
- Provide a way for users to discover the metadata file from a CSV URL and discover a CSV file from a metadata file. 
What does it mean to consume “CSV on the Web”?

- From a metadata file:
  - Process the metadata file (or fetch it from a URL).
  - Fetch the CSV files identified in the metadata.
- From a CSV URL (note that starting from a CSV alone file is a dead end):
  - Fetch the CSV file.
  - Process the HTTP headers or fallback to `/.well-known/csvw` to obtain the URL and extra metadata rules.
  - Fetch the metadata file. This can mean fetching from one to tens or more URLs.
- Validate the CSV data against the metadata rules.
- Extract contextual information, transform to RDF, etc. 
Bottomline: it is more or less easy to publish a CSVW by hand as long as you know JSON-LD, a pinch of RDF and the CSVW vocabulary. It is unreasonable to expect consuming CSVW without a robust tool.

## Context

**CSV is a poor data format** with a limited ability to set enough expectations on the structure and values contained within. Many have said that CSV is simple but I am of the opposite opinion: CSV is extremely complex from a user experience point of view; it _feels_ simple at first but eventually it lets you down. Acknowledging this puts us in a good place to understand what CSVW is trying to help with.

There are other attempts to solve the same problem such as the Frictionless Data Standard [Tabular Data Package](https://specs.frictionlessdata.io//tabular-data-package/). Check my [Dive into Frictionless Data](/notes/dive-into-frictionlessdata) for my take on it.

## Foundations

CSVW build on top of a few technical specifications, In this section I'll cover the ones I find most relevant for the complexity of CSVW.

**[IETF RFC 4180](https://tools.ietf.org/html/rfc4180)** is an attempt to formalise CSV. It aims to improve the expectations on tooling using this format.

The RFC defines a couple of topics:

- Fields are delimited by a comma character (`,`).
- Rows are delimited by a line break sequence (CRLF).
- Fields enclosed in double-quotes (`"`) allow commas, double-quotes and line breaks per field.
- Whitespace characters in fields must be preserved.
- Rows must have the same amount of fields. Empty fields are allowed.
- The content type (i.e. MIME type) is `text/csv`.
- Content should be encoded as US-ASCII or provide a `charset` attribute to the content type. 
All of the above are no more than suggestions that CSVW takes as defaults, mostly. CSVW also provides annotations to inform a parser of other dialects, such as using other field separators.

A noticeable divergence is that the default encoding for CSVW is UTF-8 which makes CSVW have defaults that are incompatible with what both RFC4180 and IANA assume. It is a mild concern given that UTF-8 is a superset of US-ASCII but still, inconsistencies always come back to bite you.

**HTTP** is, unsurprisingly, the expected vehicle to publish CSVW. It gets more entangled than expected though. For example, resetting the expected encoding or defining the absence of a header row is expected to be done via the `Content-Type` header.

Related to HTTP, CSVW depends on a handful other specifications to handle discoverability such as [IETF RFC 8288](https://tools.ietf.org/html/rfc8288), [IETF RFC 5785](https://tools.ietf.org/html/rfc5785) and [IETF RFC 8288](https://tools.ietf.org/html/rfc6570).

**[Resource Description Framework](https://www.w3.org/TR/rdf11-concepts/)** (RDF) is a requirement coming directly from the [charter](https://www.w3.org/2013/05/lcsv-charter). More specifically, the ambition was to reuse existing vocabularies where possible so “ ...] provide additional information that search engines may use to gain a better understanding of the content of the data” and “ ...] to bind the CSV content to other datasets in different formats and to Linked Data in general”.

This basically means that you **must** buy into RDF to fully benefit from the complexity you have to handle when you use CSVW otherwise you'll get lots of friction with little reward. Quite a gamble.

From the RDF requirement you get “for free” the set of datatypes defined by [XML Schema](http://www.w3.org/TR/xmlschema11-2/). The specification offers a mechanism for using other datatypes, as long as you can point to a URI. It's unclear to me on how such case would be defined and actually used by a CSVW processor though.

**[JSON-LD](https://www.w3.org/TR/json-ld/)** is the format of choice to represent the metadata file. Although it is a concrete RDF syntax it deserves its own section.

Worth noting that not any JSON-LD representation is acceptable, CSVW uses a JSON-LD dialect with a couple of restrictions. I'm afraid I don't know how much this impacts the consumption of CSVW though.

## Use cases

The CSVW Working Group compiled a set of [use cases and requirements](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/) to drive the developmemnt of the specification. This should help understanding whether my requirements align with the scope of CSVW, nice.

The first use case is worth thinking about:

> ...] the predominant format used for the exchange of Transcriptions of Records is CSV as the government departments providing the Records lack either the technology or resources to provide metadata in the XML and RDF formats ...]


So, CSV is expected due to lack of resources to do it in RDF but the specification is built on top of RDF and requires understanding and investment on it. Sounds like conflicting expectations to me.

Across the document there is a conflation of two requirements “globally unique identifiers” and “resolvable identifiers” under the “URI mapping” requirement. More evidence of a bias towards RDF. On top of that, a substantial part of the use cases already need to eventually transform to RDF, bias?.

Another important point is that the use cases are for tabular data, not strictly CSV. Well, some use cases are not directly tabular data at all but the authors took the liberty to tranform them into a normalised tabular structure. I found this a bit surprising, where is the line between use cases with a genuine need for tabular data and use cases that _could_ use tabular data or any other data model?

The requirements that are likely to fit to my common use cases are:

- [Cell microsyntax](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-CellMicrosyntax).
- [Validation](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-CsvValidation).
- [Foreign key](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-ForeignKeyReferences).
- [Primary key](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-PrimaryKey).
- [Syntactic validation](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-WellFormedCsvCheck).
- [Colocated metadata](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-ZeroEditAdditionOfSupplementaryMetadata).
- [Independent metadata](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-IndependentMetadataPublication).
- [Link from metadata to data](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-LinkFromMetadataToData).
- [Supplementary information](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-AnnotationAndSupplementaryInfo).
- [Column datatypes](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-SyntacticTypeDefinition).
- [Missing values](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-MissingValueDefinition).
- [Table groups](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-GroupingOfMultipleTables).
- [Multilingual](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-MultilingualContent). 
I can picture the potential need for these as well:

- [Transform CSV into JSON](https://www.w3.org/tr/2016/note-csvw-ucr-20160225/#R-CsvToJsonTranformation).
- [Cell delimiters other than comma](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-NonStandardCellDelimiter).
- [Text and table direction](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-RightToLeftCsvDeclaration). 
And hardly see the case for the following:

- [Transform CSV into RDF](https://www.w3.org/tr/2016/note-csvw-ucr-20160225/#R-CsvToRdfTransformation).
- [Transform CSV to a core tabular data model](https://www.w3.org/tr/2016/note-csvw-ucr-20160225/#R-CanonicalMappingInLieuOfAnnotation).
- [Property-value pair per row](https://www.w3.org/tr/2016/note-csvw-ucr-20160225/#R-SpecificationOfPropertyValuePairForEachRow).
- [External code association](https://www.w3.org/tr/2016/note-csvw-ucr-20160225/#R-AssociationOfCodeValuesWithExternalDefinitions).
- [RDF type mapping](https://www.w3.org/tr/2016/note-csvw-ucr-20160225/#R-SemanticTypeDefinition).
- [URI mapping](https://www.w3.org/tr/2016/note-csvw-ucr-20160225/#R-URIMapping).
- [Units](https://www.w3.org/tr/2016/note-csvw-ucr-20160225/#R-UnitMeasureDefinition).
- [Repeated properties](https://www.w3.org/tr/2016/note-csvw-ucr-20160225/#R-RepeatedProperties).
- [CSV subsets](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/#R-CsvAsSubsetOfLargerDataset). 
It shows that I have a bias towards normalised, self-contained, relational datasets. And by now it's clear that I do not have RDF as a priority, quite the opposite. Would the specification be significantly simpler this way? Possibly.

In any case, the document is a good place to understand why the authors of the specification made the choices they made. I wonder how many of these 25 use cases have successfully adopted CSVW.

## Data model

The data model looks fairly uncontroversial. I would've like to have a specification for the conceptual data model with no mention to parsing hints nor transformation hints (or RDF patches). It feels like it would've been a nice reusable definition other specific specifications for parsing and for transformation could extend.


<div class="figure from-dot">
<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN"
 "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<!-- Generated by graphviz version 2.48.0 (20210717.1556)
 -->
<!-- Title: g Pages: 1 -->
<svg width="554pt" height="197pt"
 viewBox="0.00 0.00 553.83 197.00" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
<g id="graph0" class="graph" transform="scale(1 1) rotate(0) translate(4 193)">
<title>g</title>
<polygon fill="transparent" stroke="transparent" points="-4,4 -4,-193 549.83,-193 549.83,4 -4,4"/>
<!-- Table -->
<g id="node1" class="node">
<title>Table</title>
<ellipse fill="none" stroke="black" cx="32.5" cy="-76" rx="30.59" ry="18"/>
<text text-anchor="middle" x="32.5" y="-72.3" font-family="Times,serif" font-size="14.00">Table</text>
</g>
<!-- Table&#45;&gt;Table -->
<g id="edge4" class="edge">
<title>Table&#45;&gt;Table</title>
<path fill="none" stroke="black" d="M18.21,-92.29C14.76,-102.39 19.53,-112 32.5,-112 40.81,-112 45.75,-108.06 47.32,-102.57"/>
<polygon fill="black" stroke="black" points="50.8,-102.1 46.79,-92.29 43.81,-102.46 50.8,-102.1"/>
<text text-anchor="middle" x="32.5" y="-115.8" font-family="Times,serif" font-size="14.00">foreign key</text>
</g>
<!-- TableGroup -->
<g id="node2" class="node">
<title>TableGroup</title>
<ellipse fill="none" stroke="black" cx="221.99" cy="-171" rx="53.89" ry="18"/>
<text text-anchor="middle" x="221.99" y="-167.3" font-family="Times,serif" font-size="14.00">TableGroup</text>
</g>
<!-- Table&#45;&gt;TableGroup -->
<g id="edge1" class="edge">
<title>Table&#45;&gt;TableGroup</title>
<path fill="none" stroke="black" d="M43.01,-93.01C51.34,-106.39 64.68,-124.46 81.05,-135 104.6,-150.17 134.24,-158.95 160.17,-164.04"/>
<polygon fill="black" stroke="black" points="159.81,-167.53 170.27,-165.88 161.06,-160.64 159.81,-167.53"/>
<text text-anchor="middle" x="115.55" y="-164.8" font-family="Times,serif" font-size="14.00">belongs</text>
</g>
<!-- Column -->
<g id="node3" class="node">
<title>Column</title>
<ellipse fill="none" stroke="black" cx="221.99" cy="-95" rx="40.09" ry="18"/>
<text text-anchor="middle" x="221.99" y="-91.3" font-family="Times,serif" font-size="14.00">Column</text>
</g>
<!-- Table&#45;&gt;Column -->
<g id="edge2" class="edge">
<title>Table&#45;&gt;Column</title>
<path fill="none" stroke="black" d="M49.18,-91.52C57.81,-98.97 69.17,-107.1 81.05,-111 113.22,-121.57 151.54,-116.02 179.94,-108.7"/>
<polygon fill="black" stroke="black" points="181.22,-111.98 189.93,-105.95 179.36,-105.23 181.22,-111.98"/>
<text text-anchor="middle" x="115.55" y="-119.8" font-family="Times,serif" font-size="14.00">has</text>
</g>
<!-- Table&#45;&gt;Column -->
<g id="edge3" class="edge">
<title>Table&#45;&gt;Column</title>
<path fill="none" stroke="black" d="M62.8,-78.97C92.01,-81.93 137.41,-86.53 172.09,-90.05"/>
<polygon fill="black" stroke="black" points="172.18,-93.57 182.48,-91.1 172.89,-86.61 172.18,-93.57"/>
<text text-anchor="middle" x="115.55" y="-90.8" font-family="Times,serif" font-size="14.00">primary key</text>
</g>
<!-- Row -->
<g id="node4" class="node">
<title>Row</title>
<ellipse fill="none" stroke="black" cx="517.89" cy="-40" rx="27.9" ry="18"/>
<text text-anchor="middle" x="517.89" y="-36.3" font-family="Times,serif" font-size="14.00">Row</text>
</g>
<!-- Table&#45;&gt;Row -->
<g id="edge5" class="edge">
<title>Table&#45;&gt;Row</title>
<path fill="none" stroke="black" d="M54.57,-63.52C62.59,-59.4 71.98,-55.28 81.05,-53 225.14,-16.76 404.47,-28.45 480.31,-35.88"/>
<polygon fill="black" stroke="black" points="480.01,-39.36 490.31,-36.89 480.71,-32.4 480.01,-39.36"/>
<text text-anchor="middle" x="315.94" y="-31.8" font-family="Times,serif" font-size="14.00">has</text>
</g>
<!-- Column&#45;&gt;Table -->
<g id="edge6" class="edge">
<title>Column&#45;&gt;Table</title>
<path fill="none" stroke="black" d="M196.98,-80.63C183.65,-73.5 166.49,-65.64 150.05,-62 120.1,-55.38 111.4,-57.63 81.05,-62 77.13,-62.57 73.1,-63.38 69.1,-64.33"/>
<polygon fill="black" stroke="black" points="68.12,-60.97 59.36,-66.93 69.92,-67.73 68.12,-60.97"/>
<text text-anchor="middle" x="115.55" y="-65.8" font-family="Times,serif" font-size="14.00">belongs</text>
</g>
<!-- Cell -->
<g id="node5" class="node">
<title>Cell</title>
<ellipse fill="none" stroke="black" cx="382.94" cy="-82" rx="27" ry="18"/>
<text text-anchor="middle" x="382.94" y="-78.3" font-family="Times,serif" font-size="14.00">Cell</text>
</g>
<!-- Column&#45;&gt;Cell -->
<g id="edge7" class="edge">
<title>Column&#45;&gt;Cell</title>
<path fill="none" stroke="black" d="M261.87,-91.82C287.45,-89.73 320.68,-87.01 345.7,-84.96"/>
<polygon fill="black" stroke="black" points="346.19,-88.44 355.88,-84.13 345.62,-81.46 346.19,-88.44"/>
<text text-anchor="middle" x="315.94" y="-91.8" font-family="Times,serif" font-size="14.00">has</text>
</g>
<!-- Row&#45;&gt;Table -->
<g id="edge8" class="edge">
<title>Row&#45;&gt;Table</title>
<path fill="none" stroke="black" d="M492.32,-32.22C485.74,-30.32 478.61,-28.43 471.94,-27 413.12,-14.39 397.94,-12.23 337.94,-8 223.42,0.08 185.92,11.69 81.05,-35 71.54,-39.23 62.51,-46.01 54.93,-52.85"/>
<polygon fill="black" stroke="black" points="52.09,-50.72 47.31,-60.17 56.94,-55.76 52.09,-50.72"/>
<text text-anchor="middle" x="315.94" y="-11.8" font-family="Times,serif" font-size="14.00">belongs</text>
</g>
<!-- Row&#45;&gt;Cell -->
<g id="edge9" class="edge">
<title>Row&#45;&gt;Cell</title>
<path fill="none" stroke="black" d="M490.24,-43.26C472.27,-45.97 448.22,-50.61 427.94,-58 422.62,-59.94 417.17,-62.44 411.99,-65.09"/>
<polygon fill="black" stroke="black" points="410.18,-62.09 403.06,-69.94 413.52,-68.25 410.18,-62.09"/>
<text text-anchor="middle" x="449.94" y="-61.8" font-family="Times,serif" font-size="14.00">has</text>
</g>
<!-- Cell&#45;&gt;Column -->
<g id="edge10" class="edge">
<title>Cell&#45;&gt;Column</title>
<path fill="none" stroke="black" d="M359.97,-72.31C353.06,-69.73 345.3,-67.3 337.94,-66 318.68,-62.61 313.13,-62.23 293.94,-66 282.02,-68.34 269.58,-72.7 258.51,-77.34"/>
<polygon fill="black" stroke="black" points="256.93,-74.21 249.18,-81.44 259.75,-80.62 256.93,-74.21"/>
<text text-anchor="middle" x="315.94" y="-69.8" font-family="Times,serif" font-size="14.00">belongs</text>
</g>
<!-- Cell&#45;&gt;Row -->
<g id="edge11" class="edge">
<title>Cell&#45;&gt;Row</title>
<path fill="none" stroke="black" d="M410.19,-82.88C428.13,-82.64 452.16,-80.62 471.94,-73 479.27,-70.18 486.46,-65.83 492.88,-61.21"/>
<polygon fill="black" stroke="black" points="495.32,-63.75 501.08,-54.85 491.03,-58.21 495.32,-63.75"/>
<text text-anchor="middle" x="449.94" y="-84.8" font-family="Times,serif" font-size="14.00">belongs</text>
</g>
</g>
</svg>
</div>

The data model covers a few types of attributes (annotations in the specification jargon).

### Structural information

Structural attributes are the ones you would expect from any Data Definition Language (DDL) where you define the columns for each table, the datatypes for each column, etc.

The line between these and parsing hints blurs a bit given that you can define your own datatypes that refine on top of other ones but other than that it feels like what I would benefit the most to reliably consume a CSV dataset as it was intended.

### Contextual annotations

Contextual annotations cover an unbounded range of needs. Provenance, spatial coverage, temporality, operational details, descriptions, licensing, etc.

To harness this massive and unknown scope the specification delegates on the extensibility mechanisms provided by RDF. In short, any attribute you might want to use is in scope as long as you can back it up with an RDF-aware vocabulary. By default you are expected to use [Dublin Core](https://dublincore.org/), [Schema.org](https://schema.org/) and [Data Catalogue](https://www.w3.org/TR/vocab-dcat-2/).

But of course, these vocabularies refine each other, overlap and your choices might not be exactly the same as mine so, in order to reason about them you need an RDF aware system. Even if you don't know what that entails, or if you can afford the costs of maintaining. Am I being too bitter here? I guess it's the consequence of once being very excited with the promises RDF and friends offered and after more than a decade only seeing a trail of deception and disenchantment. Perhaps [Solid](https://solid.github.io/specification/) is the cure. Perhaps not.

## State of the art

I found it quite difficult to find implementations so I'm sure I missed some.

| name | url | language | activity |
|-|-|-|-|
| COW: Converter for CSV on the Web | https://github.com/CLARIAH/COW | python | Active |
| csvw | https://github.com/cldf/csvw | python | Active |
| rdf-tabular | https://ruby-rdf.github.io/rdf-tabular/ | ruby | Active |
| rdf-parser-csvw | https://github.com/rdf-ext/rdf-parser-csvw | javascript | Active |
| csvlint | https://github.com/Data-Liberation-Front/csvlint.rb | ruby | Active |
| csvlint.io | https://github.com/Data-Liberation-Front/csvlint.io | web | Active |
| CSV Engine | https://github.com/ODInfoBiz/csvengine-ui | web | Active |
| csvw-validator | https://github.com/malyvoj3/csvw-validator | java | Unchanged since 2019 |
| pycsvw | https://github.com/bloomberg/pycsvw | python | Unchanged since 2017 |
| csvw-parser | https://github.com/sebneu/csvw-parser | python | Unchanged since 2016 |
| rcsvw | https://github.com/davideceolin/rcsvw | R | Unchanged since 2015 |
| csvwlib | https://github.com/Aleksander-Drozd/csvwlib | python | Unchanged since 2018 |


## Closing thoughts

It's evident that the specification (the four of them) is extremely complex and ambitious. It aims to solve a wide range of problems including structural information, contextual information, parsing hints and transformation hints.

The fact that the specification builds on top of the RDF data model leaks all over the place which makes CSVW quite expensive both tooling and cognitive wise when RDF is not part of your stack.

In summary, although CSVW aims to provide mechanisms to increase the reliability of CSV consumption, the amount of complexity you have to accept as a processor implementor or even as a casual consumer is unreasonably high.

To make this more evident, implementations are scarce, incomplete and most times abandoned.

Is CSVW a reasonable choice when you don't have or plan to have an RDF infrastructure?

My answer is no as long as there are more affordable alternatives. I guess I'll have to dig into these some time soon.

## Resources

- [CSV on the Web Working Group Charter](https://www.w3.org/2013/05/lcsv-charter).
- [CSV on the Web: A Primer](https://www.w3.org/TR/2016/NOTE-tabular-data-primer-20160225/). February 2016.
- [CSV on the Web: Use Cases and Requirements](https://www.w3.org/TR/2016/NOTE-csvw-ucr-20160225/). February 2016.
- [CSVW Implementation Report](https://w3c.github.io/csvw/tests/reports/index.html). October 2015.
- [Generating JSON from Tabular Data on the Web](https://www.w3.org/TR/2015/REC-csv2json-20151217/). December 2015.
- [Generating RDF from Tabular Data on the Web](https://www.w3.org/TR/2015/REC-csv2rdf-20151217/). December 2015.
- [IETF RFC 4180](https://tools.ietf.org/html/rfc4180). Common Format and MIME Type for Comma-Separated Values (CSV) Files. October 2005.
- [IETF RFC 5785](https://tools.ietf.org/html/rfc5785). Defining Well-Known Uniform Resource Identifiers (URIs). April 2010.
- [IETF RFC 6570](https://tools.ietf.org/html/rfc6570). URI Template. March 2012.
- [IETF RFC 8288](https://tools.ietf.org/html/rfc8288). Web Linking. October 2017.
- [JSON-LD 1.0](https://www.w3.org/TR/json-ld/). January 2014.
- [Metadata Vocabulary for Tabular Data](https://www.w3.org/TR/2015/REC-tabular-metadata-20151217/). December 2015.
- [Model for Tabular Data and Metadata on the Web](https://www.w3.org/TR/2015/REC-tabular-data-model-20151217/). December 2015.
- [W3C XML Schema Definition Language (XSD) 1.1 Part 2: Datatypes](http://www.w3.org/TR/xmlschema11-2/). April 2012.
- [RDF 1.1 Concepts and Abstract Syntax](https://www.w3.org/TR/rdf11-concepts/). February 2014.
- [Tabular Data Package](https://specs.frictionlessdata.io//tabular-data-package/). May 2017. 