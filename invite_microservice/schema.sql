DROP TABLE invites;

CREATE TABLE IF NOT EXISTS invites (
    id uuid DEFAULT uuid_generate_v4(), 
    user_id TEXT NOT NULL,
    invitor_name TEXT NOT NULL,
    invitee_name TEXT NOT NULL,
    email TEXT NOT NULL, 
    phone TEXT NOT NULL, 
    hash TEXT NOT NULL, 
    status INTEGER DEFAULT 0,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);