-- Add migration script here
CREATE TABLE users
(
    id         UUID PRIMARY KEY,
    name       VARCHAR NOT NULL,
    balance    NUMERIC(19, 4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);