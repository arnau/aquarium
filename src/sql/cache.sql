-- Copyright 2021 Arnau Siches
--
-- Licensed under the MIT license <LICENCE or http://opensource.org/licenses/MIT>.
-- This file may not be copied, modified, or distributed except
-- according to those terms.


-- Support set to prune the cache from unseen resources.
CREATE TABLE IF NOT EXISTS session_trail (
  resource_checksum text     NOT NULL,
  resource_type     text     NOT NULL,
  timestamp         datetime NOT NULL,

  UNIQUE (resource_checksum, resource_type, timestamp)
);

CREATE TABLE IF NOT EXISTS settings (
  id       text NOT NULL PRIMARY KEY,
  checksum text NOT NULL,
  blob     blob NOT NULL
);

-- TODO: Lacks the ability to express multiple representations of the same asset.
-- For example, a diagram and generated png/svg, etc.
CREATE TABLE IF NOT EXISTS asset (
  id            text NOT NULL,
  checksum      text NOT NULL,
  content_type  text NOT NULL,
  content       blob NOT NULL,

  UNIQUE (id, content_type)
);


CREATE TABLE IF NOT EXISTS tool (
  id       text NOT NULL PRIMARY KEY,
  checksum text NOT NULL,
  name     text NOT NULL,
  summary  text,
  url      text
);

CREATE TABLE IF NOT EXISTS service_account (
  id        text NOT NULL,
  person_id text NOT NULL,
  checksum  text NOT NULL,
  name      text NOT NULL,
  username  text NOT NULL,
  url       text NOT NULL,

  UNIQUE (id, person_id),
  FOREIGN KEY (person_id) REFERENCES person (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS person (
  id       text NOT NULL PRIMARY KEY,
  checksum text NOT NULL,
  name     text NOT NULL,
  guest    boolean NOT NULL
);

CREATE TABLE IF NOT EXISTS note (
  id               text NOT NULL PRIMARY KEY,
  checksum         text NOT NULL,
  title            text NOT NULL,
  summary          text NOT NULL,
  publication_date date NOT NULL,
  author_id        text NOT NULL,
  body             text NOT NULL,

  FOREIGN KEY (author_id) REFERENCES person (id)
);

CREATE TABLE IF NOT EXISTS sketch_tool (
  sketch_id text NOT NULL,
  tool_id   text NOT NULL,

  UNIQUE (sketch_id, tool_id),
  FOREIGN KEY (tool_id) REFERENCES tool (id) ON DELETE CASCADE,
  FOREIGN KEY (sketch_id) REFERENCES sketch (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS sketch (
  id               text NOT NULL PRIMARY KEY,
  checksum         text NOT NULL,
  title            text NOT NULL,
  asset_id         text NOT NULL,
  author_id        text NOT NULL,
  publication_date date NOT NULL,
  summary          text,

  FOREIGN KEY (asset_id) REFERENCES asset (id) ON DELETE CASCADE
);


CREATE TABLE IF NOT EXISTS bulletin_issue (
  id               text NOT NULL PRIMARY KEY,
  checksum         text NOT NULL,
  summary          text NOT NULL,
  publication_date date NOT NULL
);

CREATE TABLE IF NOT EXISTS bulletin_entry (
  url          text NOT NULL PRIMARY KEY,
  checksum     text NOT NULL,
  title        text NOT NULL,
  summary      text NOT NULL,
  content_type text NOT NULL,
  issue_id     text,

  FOREIGN KEY (issue_id) REFERENCES bulletin_issue (id)
);

CREATE TABLE IF NOT EXISTS bulletin_mention (
  mention_url text NOT NULL,
  entry_url   text NOT NULL,

  UNIQUE (mention_url, entry_url),
  FOREIGN KEY (entry_url) REFERENCES bulletin_entry (url) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS project (
  id         text NOT NULL PRIMARY KEY,
  checksum   text NOT NULL,
  name       text NOT NULL,
  summary    text NOT NULL,
  body       text NOT NULL,

  status     text NOT NULL,
  start_date date NOT NULL,
  end_date   date,

  source_url text
);

CREATE TABLE IF NOT EXISTS section (
  id            text NOT NULL PRIMARY KEY,
  checksum      text NOT NULL,
  title         text NOT NULL,
  resource_type text,
  body          text
);

CREATE TABLE IF NOT EXISTS entrance (
  id       text NOT NULL PRIMARY KEY,
  checksum text NOT NULL,
  body     text
);

-- The set of recent news across all relevant resources.
CREATE VIEW IF NOT EXISTS news AS
  SELECT * FROM (
    SELECT
      id,
      title,
      summary,
      'notes' AS section,
      publication_date AS date
    FROM
      note
    ORDER BY
      publication_date DESC
    LIMIT 3
  )

  UNION ALL

  SELECT * FROM (
    SELECT
      id,
      title,
      summary,
      'sketches' AS section,
      publication_date AS date
    FROM
      sketch
    ORDER BY
      publication_date DESC
    LIMIT 1
  )

  UNION ALL

  SELECT * FROM (
    SELECT
      id,
      id AS title,
      summary,
      'bulletins' AS section,
      publication_date AS date
    FROM
      bulletin_issue
    ORDER BY
      publication_date DESC
    LIMIT 1
  )

  UNION ALL

  SELECT * FROM (
    SELECT
      id,
      name AS title,
      summary,
      'projects' AS section,
      start_date AS date
    FROM
      project
    ORDER BY
      start_date DESC
    LIMIT 1
  )
ORDER BY date DESC;


-- Like news but with equal weigth for all sources.
CREATE VIEW IF NOT EXISTS feed AS
  SELECT
    id,
    title,
    summary,
    'notes' AS section,
    publication_date AS date
  FROM
    note

  UNION

  SELECT
    id,
    title,
    summary,
    'sketches' AS section,
    publication_date AS date
  FROM
    sketch

  UNION

  SELECT
    id,
    id AS title,
    summary,
    'bulletins/' || strftime('%Y', publication_date) AS section, -- TODO: Compose year.
    publication_date AS date
  FROM
    bulletin_issue

  UNION

  SELECT
    id,
    name AS title,
    summary,
    'projects' AS section,
    start_date AS date
  FROM
    project

ORDER BY date DESC;
