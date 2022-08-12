DROP generated_keys;

CREATE TABLE IF NOT EXISTS generated_keys (
    id SERIAL PRIMARY KEY, 
    user_id TEXT NOT NULL UNIQUE,
    client_id TEXT NOT NULL UNIQUE,
    client_secret TEXT NOT NULL UNIQUE 
)