-- Add migration script here
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS "user" (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT NOT NULL,
    firstname TEXT NOT NULL,
    lastname TEXT NOT NULL,
    email_verified BOOL NOT NULL,
    date_created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status TEXT NOT NULL,
    last_login TIMESTAMPTZ NULL DEFAULT NOW(),
    salt_id UUID NOT NULL
);

CREATE TABLE IF NOT EXISTS "salt" (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    salt TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "passphrase" (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL,
    passphrase TEXT NOT NULL,
    date_created TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
