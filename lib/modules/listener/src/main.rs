use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::{io::AsyncWriteExt, net::TcpListener};
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

#[derive(Debug, Serialize, Deserialize)]
struct Identify {
    username: String,
    pem: String,
}

#[tokio::main]
async fn main() {
    clear();
    let args = Args::parse();
    let addr = format!("{}:{}", args.host, args.port);
    log(
        format!("Sunucu {addr} adresinde başlatılıyor..."),
        LogType::INFO,
    );
    let listener = TcpListener::bind(&addr).await;
    if let Err(e) = listener {
        log(format!("RustIRC ana makine üzerinde bir sunucu oluşturmaya çalışırken bir hata oluştu: {e}"), LogType::ERROR);
        return;
    }
    let listener = listener.unwrap();
    log(
        format!("{addr} adresinde bir TCP sunucusu oluşturuldu. Bağlantılar için hazır."),
        LogType::OK,
    );
    loop {
        let conn = listener.accept().await;
        if let Err(e) = conn {
            log(
                format!(
                    "Dış istemciden gelen bağlantı isteği kabul edilirken bir hata oluştu: {e}"
                ),
                LogType::ERROR,
            );
            return;
        }
        let (mut socket, addr) = conn.unwrap();
        log(format!("{addr} ile ana makine arasında bir bağlantı oluşturuldu. İstemci tarafından başlangıç bayrağı bekleniyor."), LogType::STATUS);
        let identify_message = Identify {
            username: String::from("Provide your username"),
            pem: String::from("Provide the PEM for use in the handshake"),
        };
        let identify_message = serde_json::to_string(&identify_message).unwrap();
        let (reader, mut writer) = socket.split();
        let mut reader = BufReader::new(reader);
        writer.write_all(format!("Hello, stranger! You have a message from the server you tried to connect to. Please identify yourself and send your message with the protocol start flag 'FN_START' in order to receive your handshake ID.\r\nExample: |FN_START|::Identify {}\r\n\r\n", identify_message).as_bytes()).await.unwrap();
        let mut lines = String::new();
        loop {
            lines.clear();
            let bytes = reader.read_line(&mut lines).await;
            if let Err(e) = bytes {
                log(format!("Bağlantı {addr} adresli istemci tarafında gerçekleşen bir hatadan dolayı sıfırlandı: {e}"), LogType::ERROR);
                break;
            }

            lines = lines.trim().to_string();
            let identifiers: HashSet<&str> = ["FN_START", "FN_RESET"].iter().cloned().collect();
            if let Some((mut header, body)) = lines.split_once("::") {
                if let Some(header_) = header.split("|").nth(1) {
                    header = header_;
                } else {
                    println!("{addr} tarafından gönderilen mesaj bir tanımlayıcı içermiyor.");
                    if let Err(e) = writer.write_all("The message doesn't contain an identifier. Your connection will be lost.".as_bytes()).await {
                    log(format!("{addr} makinesine yanıt gönderilirken bir hata oluştu: {e}"), LogType::ERROR)
                }
                    break;
                }
                if identifiers.contains(header) {
                    let identifier = body.split_whitespace().next().unwrap_or_default();
                    let identifier_data = &body[identifier.len()..].trim();

                    match serde_json::from_str::<Identify>(identifier_data.trim()) {
                        Ok(identify) => {
                            log(
                                format!(
                                "{addr} tarafından gönderilen tanımlayıcı çözüldü: {identify:?}"
                            ),
                                LogType::STATUS,
                            );
                        }
                        Err(e) => {
                            log(
                            format!("{addr} tarafından gönderilen tanımlayıcı çözülemedi: {}. Bağlantı sonlandırılıyor..", e),
                            LogType::STATUS,
                             );

                             if let Err(e) = writer.write_all("The data is broken or unsupported. Your connection will be lost.".as_bytes()).await {
                                log(format!("{addr} makinesine yanıt gönderilirken bir hata oluştu: {e}"), LogType::ERROR);
                             }                                                
                            break;
                        }
                    }
                } else {
                    log(format!("{addr} tarafından gönderilen tanımlayıcı bozuk veya desteklenmiyor. Bağlantı sonlandırıldı."), LogType::STATUS);
                    if let Err(e) = writer
                        .write_all(
                            "Your ACK is not supported or broken. Your connection will be lost."
                                .as_bytes(),
                        )
                        .await
                    {
                        log(
                            format!("{addr} makinesine yanıt gönderilirken bir hata oluştu: {e}"),
                            LogType::ERROR,
                        )
                    }
                    break;
                }
            } else {
                log(format!("{addr} tarafından gönderilen tanımlayıcı bozuk veya desteklenmiyor. Bağlantı sonlandırıldı."), LogType::STATUS);
                if let Err(e) = writer
                    .write_all(
                        "Your ACK is not supported or broken. Your connection will be lost."
                            .as_bytes(),
                    )
                    .await
                {
                    log(
                        format!("{addr} makinesine yanıt gönderilirken bir hata oluştu: {e}"),
                        LogType::ERROR,
                    )
                }
                break;
            }
        }
    }
}

fn clear() {
    print!("\x1B[2J\x1B[1;1H");
}
