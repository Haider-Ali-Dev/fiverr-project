-- Add migration script here
CREATE TABLE images(
    path text NOT NULL,
    for_id uuid NOT NULL,
    extension text NOT NULL
);