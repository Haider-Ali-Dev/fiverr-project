-- Add migration script here
-- Fix constraint
CREATE TABLE products_owned(
    bought_at timestamp NOT NULL,
    id uuid PRIMARY KEY NOT NULL,
    product_id uuid NOT NULL,
    user_id uuid NOT NULL,
    CONSTRAINT fk_product_id FOREIGN KEY (product_id) REFERENCES products(id),
    CONSTRAINT fk_user_id FOREIGN KEY (user_id) REFERENCES users(id)
);