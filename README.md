# PDX PSU CS-510 Spring 2025 Rust Web ![Build Status](https://github.com/sashetov/recipe-server/actions/workflows/rust.yml/badge.svg)
## Table of Contents

- [PDX PSU CS-510 Spring 2025 Rust Web ![Build Status](https://github.com/sashetov/recipe-server/actions/workflows/rust.yml/badge.svg)](#pdx-psu-cs-510-spring-2025-rust-web-![build-status]https://githubcom/sashetov/recipe-server/actions/workflows/rustyml/badgesvg)
  - [`recipe-server`](#`recipe-server`)
    - [Description](#description)
    - [What was done](#what-was-done)
    - [What did not go so well/TODO](#what-did-not-go-so-well/todo)
    - [Initial setup and migrations:](#initial-setup-and-migrations:)
      - [Install SQLX CLI](#install-sqlx-cli)
      - [Rest of initial setup](#rest-of-initial-setup)
    - [Build and run it with cargo](#build-and-run-it-with-cargo)
    - [API Docs](#api-docs)
    - [Docker](#docker)
  - [Docker](#docker)
  - [`recipe-client`](#`recipe-client`)
  - [License](#license)

## `recipe-server`
### Description

Serves MIT licensed recipes. Recipes retrieved from https://github.com/Donearm/Cooking-Recipes/tree/master, which is MIT licensed and so the current license preserves that.

Much of the original inspiration for the web app itself comes from code written during the rust class, so from different branches in this repository: https://github.com/pdx-cs-rust-web/knock-knock-2, which is also MIT licensed, for similar reasons. The code is updated as updates happen in `knock-knock-2` repo, which we follow as an upstream source.

### What was done

This project mostly tracks the upstream repos:
https://github.com/pdx-cs-rust-web/knock-knock-2
https://github.com/pdx-cs-rust-web/kk2-client-yew

As a part of the CS-510 Rust Web class I would merge in the new commits as the class went along to my code.
The main difference between upstream and this app is the content is different, the fields in the json format are different and the tables are different, albeit very similar.

### What did not go so well/TODO

The project can be improved by following the fields of the recipes in https://github.com/Donearm/Cooking-Recipes/tree/master more closely and also display images.
The style and css can be improved and the site can be made responsive and to look nice under different browsers and devices.
Internationalization could be implemented.
An interface for registration, authentication and authorization to post new recipes or edit existing ones could be implemented, possily in a wiki-like manner.

### Initial setup and migrations:

#### Install SQLX CLI
```
cargo install sqlx-cli
```

#### Rest of initial setup
```
$ rm -f db/db.db
$ export DATABASE_URL="sqlite://db/db.db"
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
$ export DATABASE_URL='sqlite://db/db.db'
$ cargo build && \
   cargo clippy && \
   cargo run --release -- --init-from assets/static/recipes.json --db-uri 'sqlite://db/db.db'
```
or all of the above in one line:
```
export DATABASE_URL="sqlite://db/db.db"; rm -f db/db.db &&  sqlx database create && sqlx migrate info --source ./migrations/ &&  sqlx migrate run && cargo run --release -- --init-from assets/static/recipes.json --db-uri 'sqlite://db/db.db'
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
The README for that is located here: [Client Readme](./client/README.md)

## License

This work is made available under the "MIT License". See the file `LICENSE.txt` in this distribution for license terms.
