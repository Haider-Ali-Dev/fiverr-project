-- Add migration script here
CREATE TABLE box (
    id uuid NOT NULL PRIMARY KEY,
    price int NOT NULL,
    original_price int NOT NULL,
    listing_id uuid NOT NULL,
    CONSTRAINT fk_box_listing_id FOREIGN KEY (listing_id) REFERENCES listing (id),
    created_at timestamp NOT NULL

);