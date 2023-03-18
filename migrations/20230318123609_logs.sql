-- Add migration script here
CREATE TABLE logs(
    id uuid NOT NULL,
    user_id uuid NOT NULL,
    action text NOT NULL,
    created_at timestamp NOT NULL
);