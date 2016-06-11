extern crate toml;

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

use toml::Value;

pub fn slurp_config(path: &str) -> Value {
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

pub fn get_strings<'r>(config: &'r toml::Value, key: &'r str) -> Vec<&'r str> {
    let slice = config.lookup(key).and_then(|x| x.as_slice());
    let strings = slice.and_then(|x| x.iter().map(|c| c.as_str()).collect());
    return strings.unwrap_or(Vec::new());
}

pub fn nick_of(usermask: &str) -> &str {
    let parts: Vec<&str> = usermask.split('!').collect();
    let nick = parts[0];
    if nick.starts_with(":") {
        return &nick[1..];
    } else {
        return nick;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml;

    #[test]
    fn test_get_strings() {
        let mut parser = toml::Parser::new("[irc]\nautojoin = [\"#a\", \"#b\"]");
        let config = parser.parse().map(|x| toml::Value::Table(x)).unwrap();
        assert!(get_strings(&config, "irc.does_not_exist").is_empty());
        assert_eq!(get_strings(&config, "irc.autojoin"), ["#a", "#b"]);
    }
}
