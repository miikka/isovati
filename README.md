# isovati

**isovati** is a sketch of an IRC bot by [Miikka Koskinen](http://miikka.me/).
It's written in Rust.


## Building and running

    cargo test   # run unit tests
    cargo run    # run the IRC bot


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
