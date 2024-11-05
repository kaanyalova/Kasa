CREATE TABLE IF NOT EXISTS Thumbs (
    -- written on import so we can have placeholders
    hash TEXT NOT NULL UNIQUE,
    -- written when thumbnails are generated  
    format TEXT, 
    bytes BLOB,
    x INT NOT NULL, 
    y INT NOT NULL,
    x_max INT NOT NULL,
    y_max INT NOT NULL
);
