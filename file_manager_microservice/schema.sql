DROP TABLE files;

CREATE TABLE IF NOT EXISTS files (
    id serial,
    pid INTEGER not null,
    user_id VARCHAR(256) not null,
    name VARCHAR(256) NOT NULL,
    path VARCHAR(256) NOT NULL,
    size INTEGER NOT NULL,
    status INTEGER not null default 0,
    is_folder INTEGER NOT NULL DEFAULT 0,
    deleted INTEGER NOT NULL default 0,
    created_at timestamp(3) not null default CURRENT_TIMESTAMP,
    updated_at timestamp(3) not null default CURRENT_TIMESTAMP,

    primary key(id)
);
