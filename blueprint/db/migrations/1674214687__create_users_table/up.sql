CREATE TABLE IF NOT EXISTS users (
    id uuid PRIMARY KEY default gen_random_uuid(),
    name varchar(255) NOT NULL,
    token varchar(100) NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS users_id_idx ON users (id);
