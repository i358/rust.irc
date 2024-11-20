
use crate::commands::Command;

pub const COMMAND: Command = Command {
    name: "exit",
    usage: "/exit",
    args: &[""],
    description: "Programı sonlandırır.",
    exec: |_args, _user| {
        std::process::exit(0);
    },
};
