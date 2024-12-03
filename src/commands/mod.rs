use crate::util::{host::Host, session::Session};

pub struct Command {
    pub name: &'static str,
    pub usage: &'static str,
    pub description: &'static str,
    pub args: &'static [&'static str],
    pub exec: fn(args: Vec<String>, user: &Session, &Host),
}

pub mod help;
pub mod exit;
pub mod join;
pub mod listen;

pub fn get_commands() -> Vec<Command> {
    vec![
        help::COMMAND,
        exit::COMMAND,
        join::COMMAND,
        listen::COMMAND,
    ]
}
