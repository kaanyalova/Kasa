CREATE TABLE OcrData (
    hash TEXT NOT NULL UNIQUE,
    regions JSON NOT NULL
);