-- Add migration script here
CREATE TABLE orders
(
    id         UUID PRIMARY KEY,
    name       VARCHAR NOT NULL,
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    total      NUMERIC(19, 4) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_orders_user_id ON orders(user_id);