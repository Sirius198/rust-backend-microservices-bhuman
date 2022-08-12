DROP TABLE auth;

CREATE TABLE IF NOT EXISTS auth (
    id SERIAL PRIMARY KEY, 
    user_id TEXT NOT NULL UNIQUE, 
    access_token TEXT NOT NULL, 
    refresh_token TEXT NOT NULL, 
    provider_type TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS shopify_auth (
    id SERIAL PRIMARY KEY, 
    user_id TEXT NOT NULL UNIQUE, 
    token TEXT NOT NULL, 
    email TEXT NOT NULL
);