# HTTP Proxy

Look into Rust in order to implement a reverse HTTP proxy to bastion.

## Description

A reverse HTTP proxy is responsible for forwarding client requests to backend servers and returning the responses to clients, it often acts as an intermediary between clients and multiple backend servers. It makes it crucial for performance, reliability, and security.

More Information at [POC Findings](https://wallix.atlassian.net/wiki/spaces/PA/pages/526385176/POC+findings)

## Installation

If you want to create a executable, simply run the following command:

```bash
cargo build
```

The executable can be found at the following path: `target/debug/<binary>(.exe)`

## Quickstart

If you simply want to start the application without compiling it, run the following command (following the arguments specified in the usage):

```bash
cargo run [args]
```

## Usage

As I've included code on both the Server and Client side, use the following flags (inspect src/main.rs or follow instructions on command line through testing):

### Client

```bash
cargo run client [link]
```
or
```bash
./target/debug/http_proxy_poc client [link]
```

### Server

Run the reverse proxy:

```bash
cargo run proxy 
```
or
```bash
./target/debug/http_proxy_poc proxy
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