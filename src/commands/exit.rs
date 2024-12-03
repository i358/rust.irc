
use crate::commands::Command;

pub const COMMAND: Command = Command {
    name: "exit",
    usage: "/exit",
    args: &[""],
    description: "Programı sonlandırır.",
    exec:|_, _, _| {
        std::process::exit(0);
    },
};
