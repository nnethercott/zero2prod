-- Add migration script here
CREATE TABLE users (
  user_id uuid PRIMARY KEY,
  name TEXT NOT NULL,
  password TEXT NOT NULL
);
