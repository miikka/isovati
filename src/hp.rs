extern crate rand;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use rand::Rng;

use irc;

pub struct HP<'r> {
    name: &'r str,
    words: Vec<String>,
    rng: rand::ThreadRng
}

#[derive(Debug, PartialEq)]
pub enum Command {
    StartGame(String, char, String)
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

pub fn init<'r>(name: &'r str, path: &str) -> HP<'r> {
    let words = read_list(path);
    assert!(words.len() > 0);
    return HP { name: name, words: words, rng: rand::thread_rng() };
}

impl<'r> HP<'r> {
    fn pick_word(&mut self) -> (&String, char) {
        let word = self.rng.choose(&self.words).unwrap();
        let chars : Vec<char> = word.chars().collect();
        let chr = self.rng.choose(&chars).unwrap().clone();
        return (word, chr);
    }

    pub fn execute(&mut self, parts: &[&str]) -> Vec<Command> {
        if parts.len() == 0 {
            return vec![];
        } else if parts[0] == self.name {
            let exp = format!("se {}", &self.name);
            let (word, chr) = self.pick_word();
            return vec![Command::StartGame(word.to_string(), chr, exp)];
        }
        return vec![];
    }
}

impl Command {
    pub fn execute(self, irc: &mut irc::Irc) {
        match self {
            Command::StartGame(word, chr, exp) => {
                let _ = send!(
                    irc, "PRIVMSG Putkamon :#ohjusputka:hp \"{}\" {} \"{}\"\r\n",
                    word, chr, exp
                );
            },
        }
    }
}


mod test {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use rand;

    #[test]
    fn test_execute() {
        let mut hp = HP {
            name: "test",
            words: vec!["duck".to_string()],
            rng: rand::thread_rng()
        };
        assert_eq!(hp.execute(&[]), []);
        assert_eq!(hp.execute(&["does not exist"]), []);

        let commands = hp.execute(&["test"]);
        match commands[0] {
            Command::StartGame(ref word, _, ref exp) => {
                assert_eq!(word, "duck");
                assert_eq!(exp, "se test");
            }
        }
    }

    #[test]
    fn test_init() {
        let mut hp = init("word", "fixtures/test_word_list.txt");
        assert!(hp.execute(&["...."]).is_empty());
        assert!(!hp.execute(&["word"]).is_empty());
    }
}
