DROP TABLE users;

CREATE TABLE IF NOT EXISTS users (
    id uuid DEFAULT uuid_generate_v4(),
    user_id TEXT NOT NULL UNIQUE, 
    first_name TEXT NOT NULL, 
    last_name TEXT NOT NULL, 
    email TEXT NOT NULL,
    phone_number TEXT NOT NULL, 
    username TEXT, 
    dob TIMESTAMP(3),
    two_fator BOOLEAN DEFAULT FALSE, 
    picture TEXT, 
    gender TEXT, 
    bio TEXT, 
    user_account_type TEXT,    
    invite_users uuid[], 
    referred_by TEXT, 
    app_ids uuid[], 
    post_ids uuid[], 
    workspace_ids uuid[], 
    organization uuid[], 
    latitude REAL, 
    longitude REAL, 
    last_login_ip TEXT,
    last_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);