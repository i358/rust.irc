use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::tcp::WriteHalf;
use tokio::net::TcpListener;
mod lib;
mod util;
use lib::hex::{from_hex, to_hex};
use util::generate_uuid::generate_session_key;
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

struct Writer<'a> {
    addr: &'a SocketAddr,
    writer: &'a mut WriteHalf<'a>,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
    pem: String,
    uuid: String,
    addr: String,
    banned: bool,
}

impl User {
    fn new(u: (&String, &String, &String), uuid: String, banned: bool) -> Self {
        Self {
            username: u.0.to_string(),
            pem: u.1.to_string(),
            uuid,
            banned,
            addr: u.2.split(":").nth(0).unwrap().to_string()
        }
    }
}

impl<'a> Writer<'a> {
    pub fn new(addr: &'a SocketAddr, writer: &'a mut WriteHalf<'a>) -> Self {
        Self { addr, writer }
    }

    pub async fn write(&mut self, text: &str) {
        if let Err(e) = self.writer.write_all(format!("{}", text).as_bytes()).await {
            log(
                format!(
                    "{} makinesine yanıt gönderilirken bir hata oluştu: {}",
                    self.addr, e
                ),
                LogType::ERROR,
            );
        }
    }
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
        let mut socket_writer = Writer::new(&addr, &mut writer);

        socket_writer
            .write(&format!(
                "MSG::Hello, stranger! You have a message from the server you tried to connect to. Please identify yourself and send your message with the protocol start flag 'FN_START' in order to receive your handshake ID.\r\nExample: |FN_START|::Identify {}\r\n\r\n",
                identify_message
            ))
            .await;

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
                    log(
                        format!("{addr} tarafından gönderilen mesaj bir tanımlayıcı içermiyor."),
                        LogType::STATUS,
                    );
                    socket_writer.write("ERR::The message doesn't contain an identifier. Your connection will be lost.").await;
                    break;
                }
                if identifiers.contains(header) {
                    let identifier = body.split_whitespace().next().unwrap_or_default();
                    let identifier_data = &body[identifier.len()..].trim();

                    match identifier {
                        "Identify" => {
                            match serde_json::from_str::<Identify>(identifier_data.trim()) {
                                Ok(identify) => {
                                    let Identify { username, pem } = &identify;
                                    log(
                                        format!(
                                            r#"{addr} tarafından gönderilen tanımlayıcı çözüldü: "{username}" isimli oturum dosyası oluşturuluyor.."#
                                        ),
                                        LogType::STATUS,
                                    );
                                    socket_writer.write("OK::Connection verified. Your session is being prepared. Please wait for an ACK response before sending any messages. If you send a message before receiving the ACK, your connection will be terminated.\r\n").await;
                                    create_session((&username, &pem, &addr.to_string()));
                                }
                                Err(e) => {
                                    log(
                                        format!("{addr} tarafından gönderilen tanımlayıcı çözülemedi: {}. Bağlantı sonlandırılıyor..", e),
                                        LogType::STATUS,
                                    );
                                    socket_writer.write("ERR::The data is broken or unsupported. Your connection will be lost.").await;
                                    break;
                                }
                            }
                        }
                        _ => {
                            log(
                                format!("{addr} tarafından gönderilen tanımlayıcı {header} desteklenmiyor. Bağlantı sonlandırıldı.", header = header),
                                LogType::STATUS,
                            );
                            socket_writer
                                .write("ERR::Your identifier is not supported. Your connection will be lost.")
                                .await;
                            break;
                        }
                    }
                } else {
                    log(format!("{addr} tarafından gönderilen tanımlayıcı bozuk veya desteklenmiyor. Bağlantı sonlandırıldı."), LogType::STATUS);
                    socket_writer
                        .write("ERR::Your ACK is not supported or broken. Your connection will be lost.")
                        .await;
                    break;
                }
            } else {
                log(format!("{addr} tarafından gönderilen tanımlayıcı bozuk veya desteklenmiyor. Bağlantı sonlandırıldı."), LogType::STATUS);
                socket_writer
                    .write(
                        "ERR::Your ACK is not supported or broken. Your connection will be lost.",
                    )
                    .await;
                break;
            }
        }
    }
}

fn clear() {
    print!("\x1B[2J\x1B[1;1H");
}

fn create_session(u: (&String, &String, &String)) {
    let uuid = generate_session_key();
    let mut bytes = String::new();
    let ip = u.2.split(":").nth(0).unwrap();
    to_hex(&ip, &mut bytes);
    let user = User::new(u, uuid, false);
    let user_json = serde_json::to_string(&user).unwrap();
    println!("{user_json}");
}
