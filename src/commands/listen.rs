use crate::commands::Command;

pub const COMMAND: Command = Command {
    name: "listen",
    usage: "/listen",
    args: &[""],
    description: "Bir sunucu oluşturup yeni bağlantılar için bekler.",
    
    exec:|_, _, host| {
        let command = std::process::Command::new("cmd")
        .arg("/C")
        .arg(format!(
            "start lib/modules/server/target/debug/listener.exe -H {} -p {}",
             host.name, host.port
        ))
        .spawn();

    match command {
        Ok(child) => {
            println!("listener.exe yeni bir pencerede başlatıldı, PID: {}", child.id());
        }
        Err(e) => {
            eprintln!("listener.exe başlatılamadı: {}", e);
        }
    }
    },
};
