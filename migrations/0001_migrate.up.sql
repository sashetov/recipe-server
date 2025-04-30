-- Add up migration script here
CREATE TABLE IF NOT EXISTS recipes (
  id integer UNIQUE PRIMARY KEY NOT NULL,
  title TEXT NOT NULL,
  category TEXT NOT NULL,
  preparation TEXT NOT NULL
);
