CREATE TABLE IF NOT EXISTS RemoteMediaCache(
    hash TEXT NOT NULL UNIQUE,
    filename TEXT,
    media_type TEXT,
    video_length REAL
);