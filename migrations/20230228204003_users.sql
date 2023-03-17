-- Add migration script here
CREATE TABLE users (
    username varchar(100) NOT NULL,
    email text NOT NULL UNIQUE,
    id uuid NOT NULL PRIMARY KEY,
    password text NOT NULL,
    created_at timestamp NOT NULL,
    points int NOT NULL,
    is_superuser boolean NOT NULL,
    private_key uuid NOT NULL
);
