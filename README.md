# PDX PSU CS-586 Spring 2025 Rust Web ![Build Status](https://github.com/sashetov/recipe-server/actions/workflows/rust.yml/badge.svg)
## `recipe-server`
### Description

Serves MIT licensed recipes. Recipes retrieved from https://github.com/Donearm/Cooking-Recipes/tree/master, which is MIT licensed and so the current license preserves that.

Much of the original inspiration for the web app itself comes from code written during the rust class, so from different branches in this repository: https://github.com/pdx-cs-rust-web/knock-knock-2, which is also MIT licensed, for similar reasons. The code is updated as updates happen in `knock-knock-2` repo, which we follow as an upstream source.


### Initial setup and migrations:

#### Install SQLX CLI
```
cargo install sqlx-cli
```

#### Rest of initial setup
```
$ rm -f db.db
$ export DATABASE_URL="sqlite://db.db"
$ sqlx database create
$ sqlx migrate add -r -s migrate
$ sqlx migrate info --source ./migrations/
1/pending migrate
$ sqlx migrate run
Applied 1/migrate migrate (9.779057ms)
$ cargo sqlx prepare --check
```

### Build and run it with cargo
```
$ export DATABASE_URL='sqlite://db.db'
$ cargo build && \
   cargo clippy && \
   cargo run --release -- --init-from assets/static/recipes.json --db-uri 'sqlite://db.db'
```
or all of the above in one line:
```
export DATABASE_URL="sqlite://db.db"; rm -f db.db &&  sqlx database create && sqlx migrate info --source ./migrations/ &&  sqlx migrate run && cargo run --release -- --init-from assets/static/recipes.json --db-uri 'sqlite://db.db'
```
which I've placed in run.sh

### API Docs
Once running, you can access api docs from the /swagger-ui and /redoc URL's

### Docker
## Docker
Install docker
```
docker build -t rs .
```
You can run the image as a daemon with:

```
docker run -d -p 3000:3000 rs
```
## `recipe-client`
The README for that is located here: [Client Readme](./client/readme.md)

## License

This work is made available under the "MIT License". See the file `LICENSE.txt` in this distribution for license terms.

