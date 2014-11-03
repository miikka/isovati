extern crate openssl;
extern crate toml;

use openssl::ssl::{Tlsv1, SslContext, SslStream};
use std::io::BufferedStream;
use std::io::TcpStream;
use std::io::File;
use toml::{Table, Value};

type IrcStream = BufferedStream<SslStream<TcpStream>>;

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

#[deriving(Show)]
enum Message {
    // XXX(miikka) Does it make sense to use String instead of &str? I do not
    // understand Rust well enough to tell.
    Privmsg(String, String, String),
    Other(String),
}

fn parse_message(msg: String) -> Message {
    // IRC messages are supposed to be separated by CRLF. Drop it.
    let msg_slice = msg.as_slice().slice_to(msg.len()-2);
    let parts: Vec<&str> = msg_slice.split(' ').collect();
    match parts[1] {
        "PRIVMSG" => {
            let mut body = parts.as_slice().slice(3, parts.len()).connect(" ");
            let _ = body.remove(0); // drop the initial :
            return Privmsg(String::from_str(parts[0]),
                           String::from_str(parts[2]),
                           body);
        },
        _ => return Other(String::from_str(msg_slice)),
    }
}

fn nick_of(usermask: &String) -> &str {
    let parts: Vec<&str> = usermask.as_slice().split('!').collect();
    let nick = parts[0];
    if nick.starts_with(":") {
        return nick.slice_from(1);
    } else {
        return nick;
    }
}

fn return_chan<'r>(from: &'r String, to: &'r String) -> &'r str {
    if to.as_slice().starts_with("#") {
        return to.as_slice();
    } else {
        return nick_of(from);
    }
}

fn handle_command(irc: &mut IrcStream, msg: Message) {
    match msg {
        Privmsg(from, to, body) => {
            let parts: Vec<&str> = body.as_slice().split(' ').collect();
            match parts[0] {
                "^echo" => {
                    let rest = parts.as_slice().slice_from(1).connect(" ");
                    let _ = write!(irc, "PRIVMSG {} :{}\r\n", return_chan(&from, &to), rest);
                    let _ = irc.flush();
                },
                _ => return,
            }
        },
        _ => return,
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

    println!("Receiving now.");

    loop {
        let line = socket.read_line();
        let msg = parse_message(line.ok().unwrap());
        println!("{}", msg);
        handle_command(&mut socket, msg);
    }
}
