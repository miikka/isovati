extern crate bufstream;
extern crate openssl;
extern crate rand;
extern crate toml;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpStream;
use std::path::Path;

use bufstream::BufStream;
use openssl::ssl::{SslContext, SslMethod, SslStream};
use rand::Rng;
use toml::Value;

type IrcStream = BufStream<SslStream<TcpStream>>;

fn slurp_config(path: &str) -> Value {
    let mut config_file = File::open(&Path::new(path)).unwrap();
    let mut config_text = String::new();
    let _ = config_file.read_to_string(&mut config_text);

    let mut parser = toml::Parser::new(&config_text[..]);
    match parser.parse() {
        Some(value) => return Value::Table(value),
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

#[derive(Debug)]
enum Message {
    // XXX(miikka) Does it make sense to use String instead of &str? I do not
    // understand Rust well enough to tell.
    Privmsg(String, String, String),
    Other(String),
}

use Message::*;

fn parse_message(msg: String) -> Message {
    // IRC messages are supposed to be separated by CRLF. Drop it.
    let msg_slice = &msg[0 .. (msg.len()-2)];
    let parts: Vec<&str> = msg_slice.split(' ').collect();
    match parts[1] {
        "PRIVMSG" => {
            let mut body = parts[3..].join(" ");
            let _ = body.remove(0); // drop the initial :
            return Privmsg(String::from(parts[0]),
                           String::from(parts[2]),
                           body.clone());
        },
        _ => return Other(String::from(msg_slice)),
    }
}

fn nick_of(usermask: &String) -> &str {
    let parts: Vec<&str> = usermask[..].split('!').collect();
    let nick = parts[0];
    if nick.starts_with(":") {
        return &nick[1..];
    } else {
        return nick;
    }
}

fn return_chan<'r>(from: &'r String, to: &'r String) -> &'r str {
    if to[..].starts_with("#") {
        return &to[..];
    } else {
        return nick_of(from);
    }
}

fn handle_command(irc: &mut IrcStream, msg: Message, turhakkeet: &Vec<String>) {
    let mut rng = rand::thread_rng();

    match msg {
        Privmsg(from, to, body) => {
            let parts: Vec<&str> = body[..].split(' ').collect();
            match parts[0] {
                "^echo" => {
                    let rest = parts[1..].join(" ");
                    let _ = write!(irc, "PRIVMSG {} :{}\r\n", return_chan(&from, &to), rest);
                    let _ = irc.flush();
                },
                "^hp" => {
                    if parts[1] == "turhake" {
                        let t = rng.choose(turhakkeet).unwrap();
                        let chars: Vec<char> = t.chars().collect();
                        let c = rng.choose(&chars).unwrap();
                        let _ = write!(irc, "PRIVMSG Putkamon :#ohjusputka:hp \"{}\" {} \"se turhake\"\r\n", t, c);
                        let _ = irc.flush();
                    }
                },
                _ => return,
            }
        },
        _ => return,
    }
}

fn read_list(path: &str) -> Vec<String> {
    let f = File::open(&Path::new(path)).unwrap();
    let file = BufReader::new(&f);
    let mut lines = Vec::new();
    for line in file.lines() {
        let l = line.unwrap();
        lines.push(l.clone());
    }
    return lines;
}

pub fn get_channels<'r>(config: &'r toml::Value, key: &'r str) -> Vec<&'r str> {
    let slice = config.lookup(key).and_then(|x| x.as_slice());
    let channels = slice.and_then(|x| x.iter().map(|c| c.as_str()).collect());
    return channels.unwrap_or(Vec::new());
}

fn main() {
    let config = slurp_config("config.toml");
    let server = config.lookup("irc.server").unwrap().as_str().unwrap();
    let port = config.lookup("irc.port").unwrap().as_integer().unwrap();
    let pw = config.lookup("irc.password").unwrap().as_str().unwrap();
    let nick = config.lookup("irc.nick").unwrap().as_str().unwrap();
    let realname = config.lookup("irc.realname").unwrap().as_str().unwrap();
    let user = config.lookup("irc.username").unwrap().as_str().unwrap();

    let autojoin = get_channels(&config, "irc.autojoin");

    let turhakkeet = read_list("turhakkeet.txt");

    let raw_socket = TcpStream::connect((server, port as u16)).unwrap();
    let ssl_ctx = SslContext::new(SslMethod::Tlsv1).unwrap();
    let unbuf_socket = SslStream::new(&ssl_ctx, raw_socket).unwrap();
    let mut socket = BufStream::new(unbuf_socket);

    let _ = write!(socket, "USER {} 0 * :{}\r\n", user, realname);
    let _ = write!(socket, "PASS {}\r\n", pw);
    let _ = write!(socket, "NICK {}\r\n", nick);

    for channel in autojoin {
        let _ = write!(socket, "JOIN {}\r\n", channel);
    }

    let _ = socket.flush();

    println!("Receiving now.");

    loop {
        let mut line = String::new();
        let _ = socket.read_line(&mut line);
        let msg = parse_message(line);
        println!("{:?}", msg);
        handle_command(&mut socket, msg, &turhakkeet);
    }
}

mod test {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use toml;

    #[test]
    fn test_get_channels() {
        let mut parser = toml::Parser::new("[irc]\nautojoin = [\"#a\", \"#b\"]");
        let config = parser.parse().map(|x| toml::Value::Table(x)).unwrap();
        assert!(get_channels(&config, "irc.does_not_exist").is_empty());
        assert_eq!(get_channels(&config, "irc.autojoin"), ["#a", "#b"]);
    }
}
