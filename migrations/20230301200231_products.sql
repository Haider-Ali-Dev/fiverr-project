-- Add migration script here
CREATE TABLE products (
    box_id uuid NOT NULL,
    CONSTRAINT fk_box_id FOREIGN KEY (box_id) REFERENCES box(id),
    title text NOT NULL,
    id uuid NOT NULL PRIMARY KEY,
    description text NOT NULL,
    level int NOT NULL,
    status boolean NOT NULL,
    created_at timestamp NOT NULL,
    amount int NOT NULL,
    image text NOT NULL

);
