CREATE TABLE IF NOT EXISTS OcrData (
    id INTEGER PRIMARY KEY,
    hash TEXT NOT NULL UNIQUE,
    regions JSON NOT NULL,
    text TEXT NOT NULL DEFAULT ''
);

CREATE VIRTUAL TABLE IF NOT EXISTS OcrFTS USING fts5(
    text,
    content = 'OcrData',
    content_rowid = 'id'
);

-- keep the OcrData in sync with OcrFTS
CREATE TRIGGER OcrData_ai AFTER INSERT ON OcrData BEGIN                                                                                                                           
    INSERT INTO OcrFTS(rowid, text) VALUES (new.rowid, new.text);                                                                                                                 
END;                                                                                                                                                                              
                                                                                                                                                                                  
CREATE TRIGGER OcrData_ad AFTER DELETE ON OcrData BEGIN                                                                                                                           
    INSERT INTO OcrFTS(OcrFTS, rowid, text) VALUES('delete', old.rowid, old.text);                                                                                                
END;                                                                                                                                                                              
                                                                                                                                                                                  
CREATE TRIGGER OcrData_au AFTER UPDATE ON OcrData BEGIN                                                                                                                           
    INSERT INTO OcrFTS(OcrFTS, rowid, text) VALUES('delete', old.rowid, old.text);                                                                                                
    INSERT INTO OcrFTS(rowid, text) VALUES (new.rowid, new.text);                                                                                                                 
END;                   