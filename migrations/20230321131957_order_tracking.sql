-- Add migration script here
CREATE TABLE order_tracking (
    id uuid NOT NULL,
    status VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    product_id uuid NOT NULL,
    user_id uuid NOT NULL
);