#!/bin/bash
export DATABASE_URL="sqlite://db/db.db";
rm -f ./db/db.db &&  sqlx database create && sqlx migrate info --source ./migrations/ &&  sqlx migrate run && cargo run --release -- --init-from assets/static/recipes.json --db-uri 'sqlite://db/db.db'
