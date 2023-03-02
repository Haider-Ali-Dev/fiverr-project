-- Add migration script here
CREATE TABLE products_owned(
    bought_at timestamp NOT NULL,
    id uuid PRIMARY KEY NOT NULL,
    CONSTRAINT product_id FOREIGN KEY(id) REFERENCES products(id),
    CONSTRAINT user_id FOREIGN KEY(id) REFERENCES users(id)

);