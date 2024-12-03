use crate::commands::Command;

pub const COMMAND: Command = Command {
    name: "join",
    usage: "/join <addr>",
    args: &["addr"],
    description: "Belirli bir sunucuya katılıp sohbet etmek için kullanılır.",
    exec:|args, user, _| {
        if args.len() < 1 {
            println!("Kullanım: /join <host>:<port>");
            return;
        }

        let username = &user.name;
        let (host, port) = args[0].split_once(":").unwrap();
    

        
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
