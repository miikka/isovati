use std::collections::HashMap;
use std::io::prelude::*;

#[macro_use]
use irc;
use irc::Message;
use util;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mode {
    Op
}

pub struct Automode {
    mode_map: HashMap<(String, String), Mode>
}

#[derive(Debug, PartialEq)]
pub enum Command {
    ModeUser { user: String, mode: Mode, channel: String }
}

pub fn init(path: &str) -> Automode {
    let config = util::slurp_config(path);
    let mut mode_map = HashMap::new();

    for (channel, table) in config.as_table().unwrap() {
        let ops = util::get_strings(&table, "ops");
        for op in ops {
            mode_map.insert((channel.clone(), op.to_string()), Mode::Op);
        }
    }

    return Automode { mode_map: mode_map };
}

impl Automode {
    pub fn execute(&self, message: Message) -> Vec<Command> {
        return match message {
            Message::Join { user, channel } => {
                match self.mode_map.get(&(channel.to_string(), user.to_string())) {
                    Some(mode) => vec![Command::ModeUser {
                        user: user.to_string(),
                        channel: channel.to_string(),
                        mode: mode.clone()
                    }],
                    _ => vec![]
                }
            },
            _ => vec![]
        };
    }
}

impl Command {
    pub fn execute(self, irc: &mut irc::Irc) {
        match self {
            Command::ModeUser { user, mode, channel } => {
                let mode_char = match mode {
                    Mode::Op => "o"
                };
                let _ = send!(irc, "MODE {} +{} {}\r\n", channel, mode_char,
                              util::nick_of(&user));
            }
        }
    }
}

mod test {
    use std::collections::HashMap;
    use irc;
    use irc::Message::*;

    use super::*;

    #[test]
    fn test_execute() {
        let mut mode_map = HashMap::new();
        mode_map.insert(("#test".to_string(), "tester".to_string()), Mode::Op);

        let am = Automode { mode_map: mode_map };
        assert!(am.execute(Join{ user: "none".to_string(),
                                 channel: "#test".to_string() }).is_empty());
        assert!(am.execute(Join{ user: "tester".to_string(),
                                 channel: "#none".to_string()}).is_empty());

        let commands = am.execute(Join{ user: "tester".to_string(),
                                        channel: "#test".to_string() });
        match commands[0] {
            Command::ModeUser { ref user, mode, ref channel } => {
                assert_eq!(user, "tester");
                assert_eq!(mode, Mode::Op);
                assert_eq!(channel, "#test");
            }
        }
    }

    #[test]
    fn test_init() {
        let am = init("fixtures/test_modes.toml");
        assert!(!am.execute(Join{
            user: "test!test@example.com".to_string(), channel: "#test".to_string()
        }).is_empty());
        assert!(am.execute(Join{
            user: "fail!test@example.com".to_string(), channel: "#test".to_string()
        }).is_empty())
    }
}
