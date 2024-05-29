CREATE TABLE raw_tx (
    id SERIAL PRIMARY KEY,
    ix TEXT,
    tx TEXT,
    ts TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO raw_tx (ix, tx) VALUES ('test', 'test');