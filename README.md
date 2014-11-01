# isovati

**isovati** is a simple IRC bot. It's written in Rust.

## Building and running

    cargo build
    ./target/isovati

## Configuration

The configuration lives in `config.toml` in
[TOML](https://github.com/toml-lang/toml). Here's a template

    [irc]
    server = ""
    port = 0
    password = ""
    nick = ""
    username = ""
    realname = ""
