DROP TABLE folders;
DROP TABLE actors;
DROP TABLE video_instances;
DROP TABLE videos;
DROP TABLE audio_batch;
 
CREATE TABLE IF NOT EXISTS folders (
    id uuid DEFAULT uuid_generate_v4(), 
    user_id TEXT NOT NULL, 
    workspace_id uuid NOT NULL,
    name TEXT NOT NULL, 
    parent_videos BIGINT NOT NULL,
    generated_videos BIGINT NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS actors (
    id uuid DEFAULT uuid_generate_v4(), 
    user_id TEXT NOT NULL, 
    name TEXT NOT NULL, 
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE video_instances (
    id uuid DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    user_id TEXT NOT NULL,
    folder_id uuid NOT NULL,
    video_id uuid,
    actor_id uuid,
    audio_batch_id uuid,
    image_column_id BIGINT,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE videos (
    id uuid DEFAULT uuid_generate_v4() UNIQUE NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    length TEXT NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE audio_batch (
    id uuid DEFAULT uuid_generate_v4(),
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE generated_videos (
    id uuid DEFAULT uuid_generate_v4(),
    audio_lables TEXT[] NOT NULL,
    name TEXT NOT NULL,
    user_id TEXT NOT NULL,
    batch_id uuid NOT NULL,
    video_instance_id uuid NOT NULL,
    video_url TEXT,
    vimeo_url TEXT,
    thumbnail TEXT,
    status TEXT NOT NULL,
    vimeo_status TEXT NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE segments (
    id uuid DEFAULT uuid_generate_v4(),
    user_id TEXT NOT NULL, 
    video_instance_id uuid NOT NULL,
    prefix_time_marker_start TEXT NOT NULL,
    prefix_time_marker_end TEXT NOT NULL,
    suffix_time_marker_start TEXT NOT NULL,
    suffix_time_marker_end TEXT NOT NULL,
    audio_variable_column_id BIGINT NOT NULL,
    audio_variable_name TEXT NOT NULL,
    variable_time_marker_start TEXT NOT NULL,
    variable_time_marker_end TEXT NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE audios (
    id uuid DEFAULT uuid_generate_v4() UNIQUE NOT NULL,
    user_id TEXT NOT NULL,
    actor_id uuid NOT NULL,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    audio_length TEXT NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE audio_batch_data (
    "id" uuid DEFAULT uuid_generate_v4 (),
    "audio_batch_id" uuid NOT NULL,
    "user_id" uuid NOT NULL,
    "name" TEXT,
    "audio_id" uuid,
    "image_id" uuid,
    "row_id" BIGINT,
    "column_id" BIGINT,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("id")
);