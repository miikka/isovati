extern crate bufstream;
extern crate openssl;

use std::io::prelude::*;
use std::net::TcpStream;

use bufstream::BufStream;
use openssl::ssl::{SslContext, SslMethod, SslStream};

pub type IrcStream = BufStream<SslStream<TcpStream>>;
pub struct Irc(IrcStream);

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
