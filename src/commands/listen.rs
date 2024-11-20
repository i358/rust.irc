use crate::commands::Command;

pub const COMMAND: Command = Command {
    name: "listen",
    usage: "/listen room_password",
    args: &["password", "use_tls", "show_names"],
    description: "Bir sunucu oluşturup yeni bağlantılar için bekler.",
    
    exec:|_args, _user| {
       
    },
};
