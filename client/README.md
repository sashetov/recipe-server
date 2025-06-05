# rs-client-yew

## Table of Contents

- [About](#about)
- [Install WASM Target](#install-wasm-target)
- [Install trunk](#install-trunk)
- [Build](#build)
- [Serve](#serve)
- [License](#license)

## About

Web Client/Frontend for the Recipe Server.

Based on upstream https://github.com/pdx-cs-rust-web/kk2-client-yew

## Install WASM Target
```
rustup target add wasm32-unknown-unknown
```

## Install trunk
```
cargo install --locked trunk
```
## Build
```
cargo buld --release
```

## Serve
```
trunk serve --open
```

## License

This work is made available under the "MIT License". See the file `LICENSE.txt` in this distribution for license terms.
