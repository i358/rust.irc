use clap::Parser;
use tokio::{io::AsyncWriteExt, net::TcpListener};
use tokio::io::AsyncReadExt;
mod util;
use util::log::{log, LogType};


#[derive(Parser)]
#[command(name = "rustirc")]
#[command(author = "i358")]
#[command(version = "1.0")]
#[command(about = "Rust IRC Sunucusu")]
struct Args {
    #[arg(short = 'H', long = "hostname", default_value = "0.0.0.0")]
    host: String,
    #[arg(short = 'p', long = "port", default_value = "33363")]
    port: u16,
}

#[derive(Debug)]
struct Identify {
    username: String,
    pem: String,
}

#[tokio::main]
async fn main() {
    clear();
    let args = Args::parse();
    let addr = format!("{}:{}", args.host, args.port);
    log(format!("Sunucu {addr} adresinde başlatılıyor..."), LogType::INFO);
    let listener = TcpListener::bind(&addr).await;
    if let Err(e) = listener {
        log(format!("RustIRC ana makine üzerinde bir sunucu oluşturmaya çalışırken bir hata oluştu: {e}"), LogType::ERROR);
        return;
    }
    let listener = listener.unwrap();
    log(format!("{addr} adresinde bir TCP sunucusu oluşturuldu. Bağlantılar için hazır."), LogType::OK);    
    loop {
       let conn = listener.accept().await;
       if let Err(e) = conn {
        log(format!("Dış istemciden gelen bağlantı isteği kabul edilirken bir hata oluştu: {e}"), LogType::ERROR);
        continue;
       }
       let (mut socket, addr) = conn.unwrap();
       log(format!("{addr} ile ana makine arasında bir bağlantı oluşturuldu. İstemci tarafından başlangıç bayrağı bekleniyor."), LogType::STATUS);
       let identify_message = Identify {
        username: String::from("Provide your username"),
        pem: String::from("Provide the PEM for use in the handshake"),
    };    
       socket.write_all(format!("Hello, stranger! You have a message from the server you tried to connect to. Please identify yourself and send your message with the protocol start flag 'FN_START' in order to receive your handshake ID.\r\nExample: |FN_START|::{:?}\r\n", identify_message).as_bytes()).await.unwrap();
       let mut buff: [u8; 1024] = [0; 1024];
       let bytes = socket.read(&mut buff).await;
       if let Err(e) = bytes {
        log(format!("Bağlantı istemci tarafında gerçekleşen bir hatadan dolayı sıfırlandı: {e}"), LogType::ERROR);
        continue;
       }
       log(format!("{addr} ile TCP bağlantısı oluşturuldu. İstemci taraflı oturum açma işlemi için bekleniyor.."), LogType::STATUS);
    }
}


fn clear(){
    print!("\x1B[2J\x1B[1;1H");
}