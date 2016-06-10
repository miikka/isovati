# isovati

**isovati** is a sketch of an IRC bot by [Miikka Koskinen](http://miikka.me/).
It's written in Rust. It does not do much.


## Building and running

To build and run isovati, you need the [Rust compiler][rustc]. The rest of the
dependencies are installed by Cargo (the Rust build tool).

[rustc]: https://www.rust-lang.org/downloads.html

    cargo test   # run the unit tests
    cargo run    # run the IRC bot


## Configuration

The configuration lives in `config.toml` in
[TOML](https://github.com/toml-lang/toml). Here's a template:

    [irc]
    server = "open.ircnet.net"
    port = 6667
    password = ""
    nick = "coolbot"
    username = "coolbot"
    realname = "a really cool bot"
    autojoin = ["#cool_channel"]


## Copying

You are free to copy, modify, and distribute isovati with attribution under the
terms of the ISC license. See the LICENSE file for details.
