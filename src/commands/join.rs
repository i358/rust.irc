use crate::commands::Command;

pub const COMMAND: Command = Command {
    name: "join",
    usage: "/join <host> <port>",
    args: &["host", "port"],
    description: "Belirli bir sunucuya katılıp sohbet etmek için kullanılır.",
    exec:|args, user, _| {
        if args.len() < 2 {
            println!("Kullanım: /join <host> <port>");
            return;
        }

        let username = &user.name;
        let host = &args[0];
        let port = &args[1];

        
        let command = std::process::Command::new("cmd")
            .arg("/C")
            .arg(format!(
                "start lib/modules/client/target/debug/client.exe -H {} -p {} -u {}",
                host, port, username
            ))
            .spawn();

        match command {
            Ok(child) => {
                println!("client.exe yeni bir pencerede başlatıldı, PID: {}", child.id());
            }
            Err(e) => {
                eprintln!("client.exe başlatılamadı: {}", e);
            }
        }
    },
};
