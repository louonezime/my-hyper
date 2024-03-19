# Rust HTTP Proxy POC

Look into Rust in order to implement a reverse HTTP proxy (developed using hyper v0.14).

### Installation

If you want to create a executable, simply run the following command:

```bash
cargo build
```

The executable can be found at the following path: `target/debug/<binary>(.exe)`

### Quickstart

If you simply want to start the application without compiling it, run the following command:

```bash
cargo run
```

## Unit Testing

Run the tests:

```bash
cargo test
```

## Formatting and Linting

```bash
cargo clippy
cargo fmt
```