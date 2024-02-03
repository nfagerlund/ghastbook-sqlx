-- Add migration script here
CREATE TABLE IF NOT EXISTS visits(
    visitor TEXT PRIMARY KEY,
    count INTEGER
)
