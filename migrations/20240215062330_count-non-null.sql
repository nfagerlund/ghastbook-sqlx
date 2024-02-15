-- Add migration script here
-- ah -- https://stackoverflow.com/questions/39939445/how-to-change-a-column-from-null-to-not-null-in-sqlite3
-- fine. no alter table for me.

CREATE TABLE IF NOT EXISTS temp_visits(
    visitor TEXT PRIMARY KEY NOT NULL,
    count INTEGER NOT NULL
);
INSERT INTO temp_visits (visitor, count) SELECT visitor, count FROM visits WHERE count NOT NULL AND visitor NOT NULL;
DROP TABLE visits;
ALTER TABLE temp_visits RENAME TO visits;
