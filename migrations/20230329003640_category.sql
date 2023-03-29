-- Add migration script here
CREATE TABLE category (
    id uuid NOT NULL PRIMARY KEY,
    name text NOT NULL,
    created_at timestamp NOT NULL
);