extern crate bufstream;
extern crate openssl;

use std::io::prelude::*;
use std::net::TcpStream;

use bufstream::BufStream;
use openssl::ssl::{SslContext, SslMethod, SslStream};

pub type IrcStream = BufStream<SslStream<TcpStream>>;
pub struct Irc(IrcStream);

#[derive(Debug, Clone)]
pub enum Message {
    // XXX(miikka) Does it make sense to use String instead of &str? I do not
    // understand Rust well enough to tell.
    // XXX(miikka) Looking at automode code, String seems silly - there are
    // .to_string() calls everywhere. I *think* I should go back to &str once I
    // figure out how to handle the ownership.
    Privmsg(String, String, String),
    Join { user: String, channel: String },
    Other(String),
}

impl Irc {
    pub fn read_line(&mut self, mut line: &mut String) {
        let &mut Irc(ref mut socket) = self;
        let _ = socket.read_line(&mut line);
    }

    pub fn get_stream(&mut self) -> &mut IrcStream {
        let &mut Irc(ref mut socket) = self;
        return socket;
    }

    pub fn flush(&mut self) {
        let &mut Irc(ref mut socket) = self;
        let _ = socket.flush();
    }
}

pub struct Config<'r> {
    pub server: &'r str,
    pub port: u16,
    pub username: &'r str,
    pub password: &'r str,
    pub nick: &'r str,
    pub realname: &'r str
}

pub fn connect_ssl(config: Config) -> Irc {
    println!("Connecting to {}:{}", config.server, config.port);

    let raw_socket = TcpStream::connect((config.server, config.port)).unwrap();
    let ssl_ctx = SslContext::new(SslMethod::Tlsv1).unwrap();
    let unbuf_socket = SslStream::connect(&ssl_ctx, raw_socket).unwrap();
    let mut socket = BufStream::new(unbuf_socket);

    let _ = write!(socket, "USER {} 0 * :{}\r\n", config.username, config.realname);
    let _ = write!(socket, "PASS {}\r\n", config.password);
    let _ = write!(socket, "NICK {}\r\n", config.nick);
    let _ = socket.flush();

    return Irc(socket);
}

#[macro_export]
macro_rules! send {
    ( $irc:expr, $($x:tt)* ) => (write!($irc.get_stream(), $($x)*));
}
