DROP TABLE IF EXISTS users;
CREATE TABLE IF NOT EXISTS users (id SERIAL PRIMARY KEY, user_id TEXT NOT NULL UNIQUE, first_name TEXT NOT NULL, last_name TEXT NOT NULL, username TEXT NOT NULL, email TEXT NOT NULL, dob timestamptz NOT NULL, last_at timestamptz NOT NULL, two_fator BOOLEAN DEFAULT FALSE, picture TEXT, gender TEXT, bio TEXT, user_account_type TEXT, phone_number TEXT, invite_users TEXT, referred_by TEXT, app_ids TEXT, post_ids TEXT, workspace_ids TEXT, organization TEXT, latitude REAL, longitude REAL, last_login_ip TEXT);

DROP TABLE IF EXISTS contacts;
CREATE TABLE IF NOT EXISTS contacts (id SERIAL PRIMARY KEY, user_id TEXT NOT NULL UNIQUE, phone TEXT, email TEXT, google_contacts TEXT, outlook_contacts TEXT);

DROP TABLE IF EXISTS invites;
CREATE TABLE IF NOT EXISTS invites (id SERIAL PRIMARY KEY, user_id TEXT NOT NULL, email TEXT NOT NULL, phone TEXT NOT NULL, hash TEXT NOT NULL, used INTEGER DEFAULT 0);

DROP TABLE IF EXISTS workspaces;
CREATE TABLE IF NOT EXISTS workspaces (id SERIAL PRIMARY KEY, user_id TEXT NOT NULL, name TEXT NOT NULL, description TEXT, created_at timestamptz NOT NULL, updated_at timestamptz NOT NULL);