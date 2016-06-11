extern crate bufstream;
extern crate openssl;

use std::io::prelude::*;
use std::net::TcpStream;

use bufstream::BufStream;
use openssl::ssl::{SslContext, SslMethod, SslStream};

pub type IrcStream = BufStream<SslStream<TcpStream>>;
pub struct Irc(IrcStream);

#[derive(Debug, PartialEq)]
pub enum Message<'r> {
    Privmsg(&'r str, &'r str, &'r str),
    Join { user: &'r str, channel: &'r str },
    Ping(&'r str),
    Other(&'r str),
}

use self::Message::*;

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

pub fn parse_message(msg: &str) -> Message {
    // IRC messages are supposed to be separated by CRLF. Drop it.
    let msg_slice = &msg[0 .. (msg.len()-2)];
    let parts: Vec<&str> = msg_slice.split(' ').collect();
    if parts[0] == "PING" { return Ping(&msg_slice[6..]); }
    match parts[1] {
        "PRIVMSG" => {
            // Three spaces and a : makes 4 characters.
            let start = parts[0..3].iter()
                .map(|x| x.len()).fold(0, |sum, x| sum + x) + 4;
            return Privmsg(parts[0], parts[2], &msg_slice[start..]);
        },
        "JOIN" => {
            return Join {
                user: &parts[0][1..],
                channel: &parts[2][1..],
            }
        },
        _ => return Other(&msg_slice),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Message::*;

    #[test]
    fn test_parse_message() {
        assert_eq!(parse_message(":test!test@test PRIVMSG #test :Hello world!\r\n"),
                   Privmsg(":test!test@test", "#test", "Hello world!"));
        assert_eq!(parse_message("PING :hello world\r\n"), Ping("hello world"));
    }
}
