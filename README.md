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

### Build

```sh
cargo build
cargo test
```