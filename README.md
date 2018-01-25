# Metrics Semantics

For now it only parses the format from TOML / YAML.
The goal is to use it to derive interesting things.

## How to use

```sh
metrics --format yaml < data/example.yaml
metrics --output-format yaml < data/example.toml
```

## How to contribute

- make sure you have [rustup](rustup.rs)
- please follow <https://github.com/clevercloud/rust-guidelines>

### Build

```sh
cargo build
cargo test
```

### Webasm support

You can run the parser in the browser.

Make sure you have `cargo web` and that you're using nightly.

```sh
cargo install cargo-web
rustup update
rustup default nightly
```

Then

```sh
cd web
cargo-web build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/metrics-web.{js,wasm} public
xdg-open public/index.html
```