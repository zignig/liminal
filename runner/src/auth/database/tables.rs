// The tables for generation 

pub const ENDPOINTS: &str = " 
CREATE TABLE enpoints IF NOT EXISTS (
    id INTEGER PRIMARY KEY,
    endpoint TEXT NOT NULL UNIQUE,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    status INTEGER,
    rcan TEXT,
";

