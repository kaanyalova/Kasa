-- For what these tables are for see kasa_core/db/schema.rs

CREATE TABLE Media (
    hash TEXT NOT NULL UNIQUE,
    thumb_path TEXT,
    media_type TEXT,
    filesize NUMBER,
    mime TEXT NOT NULL,
    thumbnail_x INT ,
    thumbnail_y INT,
    time_added INT,
    has_file_ref BOOLEAN
);


CREATE TABLE Image (
    hash NOT NULL,
    -- format TEXT NOT NULL,
    resolution_x INT NOT NULL,
    resolution_y INT NOT NULL
);



CREATE VIRTUAL TABLE Tag USING fts5 (
    name
);


CREATE TABLE Path (
    hash TEXT NOT NULL,
    path TEXT NOT NULL,
    imported_from TEXT,
    UNIQUE (hash, path)
);


CREATE TABLE HashTagPair (
    hash TEXT NOT NULL,
    tag_name TEXT NOT NULL,
    source TEXT,
    source_type TEXT,
    UNIQUE (hash, tag_name)
);

-- wtf tag search goes from 0.7s to 0.003
CREATE INDEX idx_hash_tag_pair__tag_name ON HashTagPair(tag_name);

CREATE TABLE RawTagsField (
    hash TEXT NOT NULL UNIQUE,
    _text TEXT
);


CREATE TABLE TagDetail (
    name TEXT NOT NULL,
    delete_on_no_references_left BOOLEAN NOT NULL DEFAULT true,
    color TEXT,
    _group TEXT,
    override_group_color BOOLEAN NOT NULL DEFAULT false
);


CREATE TABLE TagGroup (
    name TEXT NOT NULL,
    color TEXT
); 

CREATE TABLE IndexSource (
    path TEXT NOT NULL
);


CREATE TABLE VirtualIndexSource (
    path TEXT NOT NULL
);