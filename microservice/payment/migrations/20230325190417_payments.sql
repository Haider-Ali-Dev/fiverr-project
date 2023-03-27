-- Add migration script here
CREATE TABLE payments (
    s_id serial NOT NULL,
    id uuid NOT NULL,
    created_at timestamp NOT NULL,
    amount int NOT NULL,
    user_id uuid NOT NULL,
    user_email text NOT NULL,
    tty_points TEXT NOT NULL
);