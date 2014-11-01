extern crate openssl;
extern crate toml;
extern crate collections;

use openssl::ssl::{Tlsv1, SslContext, SslStream};
use std::io::BufferedStream;
use std::io::TcpStream;
use std::io::File;
use toml::{Table, Value};

fn slurp_config(path: &str) -> Value {
    let config_text = File::open(&Path::new(path)).read_to_string().unwrap();
    let mut parser = toml::Parser::new(config_text.as_slice());
    match parser.parse() {
        Some(value) => return Table(value),
        None => {
            println!("Parsing {} failed.", path);
            for error in parser.errors.iter() {
                let (ll, lc) = parser.to_linecol(error.lo);
                let (hl, hc) = parser.to_linecol(error.hi);
                println!("{}({}:{}-{}:{}): {}", path, ll+1, lc+1, hl+1, hc+1, error.desc);
            }
            panic!("Parsing config failed.");
        },
    }
}

fn main() {
    let config = slurp_config("config.toml");
    let server = config.lookup("irc.server").unwrap().as_str().unwrap();
    let port = config.lookup("irc.port").unwrap().as_integer().unwrap().to_u16().unwrap();
    let pw = config.lookup("irc.password").unwrap().as_str().unwrap();
    let nick = config.lookup("irc.nick").unwrap().as_str().unwrap();
    let realname = config.lookup("irc.realname").unwrap().as_str().unwrap();
    let user = config.lookup("irc.username").unwrap().as_str().unwrap();

    let raw_socket = TcpStream::connect(server, port).unwrap();
    let ssl_ctx = SslContext::new(Tlsv1).unwrap();
    let unbuf_socket = SslStream::new(&ssl_ctx, raw_socket).unwrap();
    let mut socket = BufferedStream::new(unbuf_socket);

    let _ = write!(socket, "USER {} 0 * :{}\r\n", user, realname);
    let _ = write!(socket, "PASS {}\r\n", pw);
    let _ = write!(socket, "NICK {}\r\n", nick);
    let _ = socket.flush();

    let _ = write!(socket, "PRIVMSG #ohjusputka :tutturuu\r\n");
    let _ = socket.flush();

    println!("Receiving now.");

    for line in socket.lines() {
        print!("{}", line.ok().unwrap());
    }
}
