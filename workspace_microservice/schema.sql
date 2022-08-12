CREATE TABLE IF NOT EXISTS workspaces (
    id SERIAL PRIMARY KEY,
    workspace_id uuid NOT NULL DEFAULT uuid_generate_v4(), 
    user_id TEXT NOT NULL, 
    name TEXT NOT NULL, 
    description TEXT, 
    role TEXT NOT NULL, 
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP
);