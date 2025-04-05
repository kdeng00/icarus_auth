-- Add migration script here
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS "user" (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    date_created TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
