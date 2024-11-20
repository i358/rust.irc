use crate::commands::Command;

pub const COMMAND: Command = Command {
    name: "join",
    usage: "/join <host> <port> <kanal_adi>",
    args: &["host", "port", "kanal_adi"],
    description: "Belirli bir kanala katılıp sohbet etmek için kullanılır.",
    exec: |_args, _user| {

    },
};
