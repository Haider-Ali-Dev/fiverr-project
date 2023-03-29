-- Add migration script here
CREATE TABLE listing (
    id uuid NOT NULL PRIMARY KEY,
    title text NOT NULL,
    created_at timestamp NOT NULL,
    tty text NOT NULL,
    description text NOT NULL,
    category_id uuid v
);
