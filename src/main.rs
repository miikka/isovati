extern crate bufstream;
extern crate openssl;
extern crate rand;
extern crate toml;

use std::io::prelude::*;

#[macro_use]
mod irc;
mod hp;
mod automode;
mod util;

use irc::Message;
use irc::Message::*;

fn return_chan<'r>(from: &'r str, to: &'r str) -> &'r str {
    if to[..].starts_with("#") {
        return to;
    } else {
        return util::nick_of(from);
    }
}

fn handle_command(irc: &mut irc::Irc, msg: Message, hp: &mut hp::HP,
                  automode: &automode::Automode) {
    match msg {
        Privmsg(from, to, body) => {
            let parts: Vec<&str> = body[..].split(' ').collect();
            match parts[0] {
                "^echo" => {
                    let rest = parts[1..].join(" ");
                    let _ = send!(irc, "PRIVMSG {} :{}\r\n", return_chan(from, to), rest);
                },
                "^hp" => {
                    let commands = hp.execute(&parts[1 ..]);
                    for command in commands {
                        command.execute(irc);
                    }
                },
                _ => return,
            }
        },
        Join { .. } => {
            let commands = automode.execute(msg);
            for command in commands {
                command.execute(irc);
            }
        },
        Ping(identifier) => {
            let _ = send!(irc, "PONG :{}\r\n", identifier);
        },
        _ => return,
    }
    let _ = irc.flush();
}

fn main() {
    let config_path = "config.toml";
    println!("Loading configuration from {}.", config_path);
    let config = util::slurp_config(config_path);

    let server = config.lookup("irc.server").unwrap().as_str().unwrap();
    let port = config.lookup("irc.port").unwrap().as_integer().unwrap();
    let pw = config.lookup("irc.password").unwrap().as_str().unwrap();
    let nick = config.lookup("irc.nick").unwrap().as_str().unwrap();
    let realname = config.lookup("irc.realname").unwrap().as_str().unwrap();
    let user = config.lookup("irc.username").unwrap().as_str().unwrap();

    let autojoin = util::get_strings(&config, "irc.autojoin");
    let automode = automode::init("conf/automode.toml");
    let mut hp = hp::init("turhake", "turhakkeet.txt");

    let irc_config = irc::Config {
        server: server, port: port as u16, username: user, password: pw,
        nick: nick, realname: realname
    };
    let mut irc_handle = irc::connect_ssl(irc_config);

    for channel in autojoin {
        let _ = send!(irc_handle, "JOIN {}\r\n", channel);
    }

    irc_handle.flush();

    println!("Receiving now.");

    loop {
        let mut line = String::new();
        let _ = irc_handle.read_line(&mut line);
        let msg = irc::parse_message(&line[..]);
        println!("{:?}", msg);
        handle_command(&mut irc_handle, msg, &mut hp, &automode);
    }
}
