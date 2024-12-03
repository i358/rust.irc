mod commands;
use commands::Command;
use std::collections::HashMap;
mod lib;
mod util;
use anyhow::{Context, Result};
use colored::*;
use figlet_rs::FIGfont;
use ini::Ini;
use lib::hex::{from_hex, to_hex};
use rand::Rng;
use std::fs;
use std::io::{self, Write};
use util::host::Host;
use util::session::{Privacy, Session};

fn main() {
    if let Err(e) = run() {
        eprintln!("Bir hata oluştu: {}", e);
        wait_for_user(); 
    }
}

fn run() -> Result<()> {
    println!("Ayarlar alınıyor...");
    if fs::metadata("config/session.ini").is_ok() {
        println!("Oturum bilgileri alınıyor...");
        let config = Ini::load_from_file("config/session.ini")
            .with_context(|| "config/session.ini dosyası yüklenemedi.")?;

        if let Some(dat_file) = config.get_from(Some("Locations"), "dat") {
            if fs::metadata(dat_file).is_ok() {
                println!("Bir adet oturum kaydı bulundu, oturum geri yükleniyor...");
                start_session(dat_file)?;
            } else {
                println!("Önceden oluşturulmuş bir oturum kaydı bulunamadı. Yeni bir oturum oluşturuluyor...");
                create_session(dat_file)?;
            }
        } else {
            println!("Ayar dosyasında gerekli anahtar bulunamadı. Dosya yeniden oluşturuluyor...");
            fs::remove_file("config/session.ini")
                .with_context(|| "config/session.ini dosyası silinemedi.")?;
            run()?;
        }
    } else {
        println!("Ayarları içeren dosya bulunamadı, yenisi oluşturuluyor...");
        create_config_file()?;
        println!("Ayar dosyası oluşturuldu, program yeniden başlatılıyor...");
        run()?;
    }

    Ok(())
}

fn create_config_file() -> Result<()> {
    let config_dir = "config";
    if !fs::metadata(config_dir).is_ok() {
        fs::create_dir_all(config_dir)
            .with_context(|| format!("Dizin oluşturulamadı: {config_dir}"))?;
    }

    let mut sets = Ini::new();
    sets.with_section(Some("Locations"))
        .set("dat", "session.dat");

    sets.write_to_file("config/session.ini")
        .with_context(|| "Ayar dosyası oluşturulurken bir hata oluştu.")?;
    Ok(())
}

fn create_session(dat_file: &str) -> Result<()> {
    loop {
        clear();
        println!("Yeni bir oturum oluşturmak için bazı bilgilere ihtiyacımız var.");
        print!("\nMesajlaşırken kullanacağınız rumuzu yazın (sonradan değiştirebilir ve gizleyebilirsiniz): > ");
        io::stdout().flush().expect("An unexpected error occurred.");
        let mut username = String::new();
        io::stdin()
            .read_line(&mut username)
            .expect("An error occurred");
        username = username.trim().to_string();
        if username.is_empty() {
            clear();
            continue;
        }
        clear();
        set_privacy_choices(&username, dat_file)?;
        break;
    }
    Ok(())
}

fn set_privacy_choices(username: &String, dat_file: &str) -> Result<()> {
    loop {
        println!("\nMerhaba, {username}. Gizlilik tercihlerinizi özelleştirmeniz gerekecek, aşağıdaki gizlilik tercihlerinden iki adet seçin. (Seçimleri ayırmak için virgül kullanın. Ayarları sonradan değiştirebilirsiniz):\n");
        println!("(a) Kullanıcı adını göster.");
        println!("(b) Gelen bütün bağlantı isteklerine izin ver.");
        println!("(c) Kullanıcını adını gizle.");
        println!("(d) Gelen bütün bağlantı isteklerini reddet.");
        print!("\n(ex: a,b): > ");
        let mut choices = String::new();
        io::stdout().flush().expect("An error occurred");
        io::stdin()
            .read_line(&mut choices)
            .expect("An error occurred");
        let mut privacy: Vec<Privacy> = vec![];
        if choices.trim().split(',').count() < 2 {
            clear();
            println!("En az 2 tercih yapmanız gerekiyor.");
            continue;
        }
        if choices.trim().contains("a") && choices.trim().contains("c") {
            clear();
            println!("Aynı anda iki zıt seçimi yapamazsınız.");
            continue;
        }
        if choices.trim().contains("b") && choices.trim().contains("d") {
            clear();
            println!("Aynı anda iki zıt seçimi yapamazsınız.");
            continue;
        }
        choices = choices.trim().to_string();
        for choice in choices.trim().split(',') {
            match choice {
                "a" => privacy.push(Privacy::ShowName),
                "b" => privacy.push(Privacy::AcceptConnections),
                "c" => privacy.push(Privacy::HideName),
                "d" => privacy.push(Privacy::RefuseConnections),
                _ => {
                    clear();
                    println!("Böyle bir seçenek kapsamda bulunamadı.");
                }
            }
        }
        clear();
        if privacy.is_empty() {
            continue;
        }
        println!("Ayarlarınız uygulanıyor ve oturum kaydediliyor..");
        save_choices(username, privacy, dat_file)?;
        break;
    }
    Ok(())
}

fn save_choices(username: &str, privacy: Vec<Privacy>, dat_file: &str) -> Result<()> {
    let session = Session::new(username, privacy);
    let session_json = serde_json::to_string(&session)
        .context("Oturum verileri serileştirilirken bir hata oluştu.")?;
    let mut bytes = String::new();
    to_hex(session_json, &mut bytes);
    fs::write(dat_file, bytes).context("Oturum dosyasına yazılırken bir hata oluştu.")?;
    clear();
    println!("Ayarlarınız uygulandı. Oturumunuz başlatılıyor.");
    start_session(dat_file)?;
    Ok(())
}

fn start_session(dat_file: &str) -> Result<()> {
    let bytes = fs::read_to_string(dat_file)
        .with_context(|| "Oturum dosyası okunurken bir hata oluştu.")?;
    let session_json = from_hex(&bytes)
        .map_err(|e| anyhow::anyhow!("Oturum dosyası çözülürken bir hata oluştu: {}", e))?;
    let session: Session = serde_json::from_str(&session_json)
        .context("Oturum verileri yüklenirken bir hata oluştu.")?;

    if session.name.is_empty() || session.privacy_options.len() < 2 {
        invalidate_session()?;
    }

    app(session);
    Ok(())
}

fn invalidate_session() -> Result<()> {
    println!("Oturumunuzun geçerli olmadığından oturum kapatılıyor.");
    fs::remove_file("session.dat")
        .context("Geçersiz oturum dosyası silinirken bir hata oluştu.")?;
    Ok(())
}

fn clear() {
    print!("\x1B[2J\x1B[1;1H");
}

#[tokio::main]
async fn app(user: Session) {
    clear();
    let standard_font = FIGfont::standard().unwrap();

    let figure = standard_font.convert("RustIRC");

    if let Some(art) = figure {
        for line in art.to_string().lines() {
            println!("{}", line.yellow().bold());
        }
    } else {
        println!("{}", "Beklenmedik bir hata oluştu.".red());
    }
    println!(
        "{}",
        format!(
            "\nRustIRC'ye hoş geldin, {}. Komut listesini görüntülemek için /help yazabilirsin.\n",
            user.name
        )
        .bold()
    );
    let port: u16 = rand::thread_rng().gen_range(1000..65535);
    let host: Host = Host::new("127.0.0.1", port);
    let commands: HashMap<String, Command> = commands::get_commands()
        .into_iter()
        .map(|cmd| (cmd.name.to_string(), cmd))
        .collect();
    loop {
        let mut command = String::new();
        print!(
            "{}",
            format!("\n{}@[{}:{}] > ", user.name, host.name, host.port)
                .blue()
                .bold()
        );
        io::stdout().flush().expect("An error occurred");
        io::stdin()
            .read_line(&mut command)
            .expect("An error occurred");
        command = command.trim().to_string();
        if command.starts_with("/") {
            let parts: Vec<String> = command[1..]
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            if let Some((cmd_name, args)) = parts.split_first() {
                if let Some(cmd) = commands.get(cmd_name) {
                    println!("");
                    (cmd.exec)(args.to_vec(), &user, &host);
                } else {
                    println!("Bilinmeyen komut: /{}", cmd_name);
                }
            }
        } else {
           match command.as_str() {
               "clear" => clear(),
               "cls" => clear(),
               "exit" => std::process::exit(0),
               _ =>  println!("Böyle bir komut ya da sözdizimi bulunamadı.")
           }
        }
    }
}

fn wait_for_user() {
    println!("Devam etmek için Enter tuşuna basın...");
    let mut _input = String::new();
    io::stdin().read_line(&mut _input).unwrap();
}
